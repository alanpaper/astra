---
name: project-guide
description: 星野 (Astra) 项目的完整架构指南。说明项目目录结构、前后端文件职责、Tauri 命令清单、状态管理方式和开发约定。当需要了解项目全貌、定位文件、扩展功能或修改架构时使用此 skill。
---

# 星野 (Astra) 项目架构指南

## 项目概览

**星野** 是一个桌面应用，定位为「项目工作空间管理工具 + AI 对话助手」。

| 项 | 值 |
|----|-----|
| 技术栈 | SvelteKit (SPA) + Tauri 2 |
| 语言 | TypeScript (前端) + Rust (后端) |
| 包管理 | pnpm |
| 构建 | `pnpm build` |
| 开发 | `pnpm dev` |
| 类型检查 | `pnpm check` |
| Tauri | `pnpm tauri dev` / `pnpm tauri build` |

核心功能页面：
1. **工作空间** (`/`) — 管理工作空间目录、扫描项目、创建/打开项目、项目中 node_modules 管理
2. **对话** (`/chat`) — 与 AI 模型对话，工具栏在顶部标题栏
3. **工具箱** (`/tools`) — 下载管理、node_modules 批量清理等工具

---

## 目录结构

```
astra/
├── src/                          # 前端源码
│   ├── lib/                      # 共享模块
│   │   ├── theme.svelte.ts       # 主题管理（亮/暗 + 渐变背景）
│   │   ├── workspace.svelte.ts   # 全局工作空间状态
│   │   ├── nm-store.svelte.ts    # node_modules 扫描/清理全局状态
│   │   ├── chat-state.svelte.ts  # 对话页工具栏共享状态
│   │   ├── ChatToolbar.svelte    # 对话页工具栏组件（在标题栏渲染）
│   ├── routes/                   # SvelteKit 路由
│   │   ├── +layout.svelte        # 根布局：侧边栏 + 可拖拽标题栏（loading 常驻右侧）
│   │   ├── +layout.ts            # SPA 配置 (ssr = false)
│   │   ├── +page.svelte          # 工作空间页面 (/)
│   │   ├── chat/                 # 对话页面 (/chat)
│   │   │   ├── +page.svelte
│   │   │   └── MarkdownMessage.svelte
│   │   ├── tools/                # 工具箱 (/tools, /tools/nm, /tools/downloader)
│   │   ├── models/               # 本地模型管理
│   │   ├── providers/            # API Provider 配置
│   │   ├── settings/             # 设置页面
│   │   └── skills/               # Skills 管理
│   ├── styles/
│   │   └── global.css            # 全局 CSS 变量 + spin keyframes
│   └── app.html
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── main.rs               # 入口
│   │   ├── lib.rs                # 主库：工作空间、项目、编辑器、模型、Skills、node_modules
│   │   ├── chat.rs               # AI 对话（流式 SSE）
│   │   ├── chat_sessions.rs      # 对话会话持久化
│   │   ├── command_runner.rs     # Shell 命令执行
│   │   ├── downloader.rs         # 文件下载（断点续传）
│   │   └── providers.rs          # API Provider CRUD
│   ├── capabilities/             # Tauri 权限配置
│   ├── tauri.conf.json
│   └── Cargo.toml
└── package.json
```

---

## 前端文件职责

### `src/lib/workspace.svelte.ts` — 全局工作空间状态

跨页面共享的工作空间数据，使用 Svelte 5 `$state` 实现。

```typescript
class WorkspaceStore {
  projects     // 当前工作空间的项目列表
  workspaces   // 所有工作空间配置
  activePath   // 活动工作空间路径
  activeName   // 活动工作空间名称
  editor       // 编辑器配置 { name, command }
  loading
  // methods
  loadFromSettings()
  scanProjects(path)
  switchWorkspace(path)
  findProjectByName(name)
}
export const workspaceStore
```

### `src/lib/nm-store.svelte.ts` — node_modules 全局状态

独立于页面生命周期的响应式 store：

```typescript
export const nm = $state({
  scanning: false, cleaning: false,
  error: '', progress: '',
  activePath: null, cancelled: false,
})
export const nmCache = new Map()  // 扫描结果缓存

startScan(path)    // 异步扫描，完成后写缓存
stopScan()         // 设置取消标记
cleanSingle(path)  // 删除单个
cleanAll(paths)    // 批量删除
```

**注意**：`startScan` 在后台执行，即使切换页面也会完成。标题栏通过 `layout.svelte` 的 `$effect` 同步 `nm.scanning/cleaning/progress` 显示状态。

### `src/lib/chat-state.svelte.ts` — 对话页工具栏状态

```typescript
export const toolbarState = $state({
  sourceType, selectedProviderId,
  selectedModelPort, overrideModelName,
  isSending, showSettings,
  providers, providerModels, runningModels,
  modelsLoading, sessionsCount,
  onClear, onToggleSidebar,  // 由 chat 页面挂载的回调
})
```

工具栏组件 `ChatToolbar.svelte` 读取此状态，layout 在 `/chat` 路由时渲染。

### `src/routes/+layout.svelte` — 根布局

- 可拖拽顶部标题栏，左侧 70px padding 留给 macOS 三按钮
- 标题栏布局：`[macOS按钮区] [星野] [loading指示器(常驻右侧)] [titlebar-slot]`
  - `titlebar-slot` 在 `/chat` 路由时渲染 `<ChatToolbar>` 组件
  - loading 指示器始终在 DOM 中，通过 `opacity` 控制显隐
- 左侧可折叠侧边栏（工作空间、对话、工具箱、设置）
- 全局快捷键：`Cmd+,` 打开设置，`Cmd+W` 最小化到托盘
- 移动端汉堡菜单
- 全局响应式状态：通过 `$effect()` 同步 `nm-store` 的 `nm.scanning/cleaning/progress`

### `src/routes/+page.svelte` — 工作空间页面

项目卡片列表、搜索、详情视图、新建项目。

核心函数：`loadAndScan`, `switchWorkspace`, `scanWorkspace`, `openProject`, `createNewProject`, `showDetail`, `openEditorForPath`

**项目卡片**：每张卡片底部显示项目源码大小（通过 `get_project_size` 计算，跳过 node_modules/.git/target 等非源码目录）

**node_modules 清理**（项目详情内）：
- 进入项目详情时从 `nmCache` 读取缓存，有缓存直接展示
- 手动点击「扫描 node_modules」触发 `scan_node_modules` 命令
- 扫描结果存入 `nmCache`（模块级 Map，跨页面导航不丢失）
- 每项显示路径和大小，悬停显示删除按钮（点击需确认弹窗）
- 一键清理全部按钮
- pnpm 项目（检测 pnpm-lock.yaml）正常显示和删除
- 操作时标题栏右侧显示 spinner + 进度文字 + 停止按钮

### `src/routes/chat/+page.svelte` — 对话页面（核心功能页）

这是项目最复杂的页面。工具栏（模型/Provider 选择、设置、清空、历史）已提取到 `ChatToolbar.svelte` 组件，通过 layout 的 `titlebar-slot` 渲染在顶部标题栏。

页面包含以下子系统：

1. **来源切换**：通过 `chat-state.svelte.ts` 的 `toolbarState.sourceType` 控制 Provider / 本地模型
2. **流式对话**：监听 Tauri 事件 `chat-chunk` / `chat-chunk-reasoning` / `chat-done` / `chat-error`
3. **会话历史**：右侧抽屉，搜索过滤
4. **@ 项目提及**：
   - 输入 `@` 触发弹出框（正则检测 `(?:^|\s)@([\w\u4e00-\u9fff.-]*)$`）
   - 实时搜索全部项目，弹出框支持 `↑↓` 导航 + `Enter` 确认 + `Esc` 关闭
   - 滚动跟随：`scrollIntoView({ block: 'nearest' })`
   - 选中后插入 `@项目名 ` 格式
5. **Workspace 联动**：
   - `buildWorkspaceContext()` — 增强系统提示，注入项目列表和 action 协议
   - `parseMentions(text)` — 解析用户消息中的 @ 项目名
   - `handleSend` 把提及项目路径追加到上下文
6. **Action 链接触发命令**：
   - AI 回复中可输出 `[🚀 打开](action://open_project?path=xxx)` markdown 链接
   - 新消息中的 action 链接**自动执行**（无需点击），通过 `data-auto-executed` 标记防重复
   - `handleActionClick` 拦截点击 → 调用 Tauri 命令（如 `open_in_editor`）
   - 三种状态：`action-executed`（执行中）、`action-done`（成功，pointer-events: none）、`action-error`（失败）
   - 防重复点击
7. **代码块一键执行**：
   - AI 输出的 `bash`/`sh`/`shell` 代码块自动显示 **▶ 执行** 按钮
   - 点击后弹出确认框，用户点击 **允许执行** 后运行
   - 通过 `MarkdownMessage.svelte` 的 `handleExecClick` → `executeCommand` 流程处理
   - 执行结果实时显示在代码块下方（`command-stdout` / `command-stderr` 事件）
   - 响应式确认机制，防止误操作
8. **`isFresh` 标记**：新消息标记 `isFresh = true`，历史消息 `isFresh = false`，控制 action 和自动执行行为

### `src/routes/chat/MarkdownMessage.svelte` — Markdown 渲染 + 命令执行

使用 `marked` + `highlight.js` 代码高亮 + 代码块复制按钮。
action 链接通过此组件渲染为 `<a href="action://...">`。

**核心职责扩展（已超越纯渲染器）**：
- 渲染后自动扫描并执行 `action://open_project` 链接（仅新消息）
- 识别 `bash`/`sh`/`shell` 代码块，渲染 **▶ 执行** 按钮
- 内置确认弹窗（显示完整命令内容）
- 通过 Tauri 事件 `command-stdout` / `command-stderr` / `command-done` / `command-error` 接收实时执行结果
- 执行输出展示在代码块下方，支持关闭
- 防重复执行标记（`data-auto-executed` + `executions[]` 状态追踪）

### 其他路由

- `/settings` — 工作空间配置、默认编辑器、扫描深度
- `/providers` + `/providers/[id]` — API Provider CRUD（OpenAI 兼容）
- `/models` + `/models/[id]` — 本地模型配置（支持启动 llama.cpp 服务器）
- `/skills` + `/skills/[name]` — 浏览/删除系统中的 Skills

---

## 后端文件职责 (Rust)

### `src-tauri/src/lib.rs` — 主库

包含绝大多数 Tauri 命令（大多数已改为 `async` + `spawn_blocking` 避免阻塞 UI）：

**工作空间 / 项目**
- `scan_workspace(path)` — `async`，递归扫描工作空间下的项目（检测 README.md，计算源码大小）
- `get_project_detail(path)` — 项目详情（子项目、git remote、README 预览）
- `create_project` — 创建项目目录

**node_modules 管理**
- `scan_node_modules(workspace_path, max_depth)` — `async`，扫描目录下所有 node_modules（含大小和 pnpm 检测）
- `clean_node_modules(paths)` — `async`，批量删除 node_modules
- `get_settings` / `save_settings` — 应用配置读写
- `add_workspace` / `remove_workspace` / `set_active_workspace`

**编辑器**
- `open_in_editor(path, editor_command)` — 用编辑器打开（兼容 PATH）
- `get_preset_editors` — 预置编辑器列表

**本地模型服务器**
- `start_llama_server` / `stop_llama_server`
- `list_running_servers` / `check_server_status`
- `list_model_configs` / `save_model_config` / `delete_model_config`

**Skills**
- `list_skills` / `delete_skill` / `read_skill_doc` — 解析 SKILL.md frontmatter

**命令执行**（见 `command_runner.rs`）
- `run_command(command, cwd, timeout_secs)` — 执行 shell 命令，流式输出
  - 事件：`command-stdout` / `command-stderr` / `command-done` / `command-error`
  - 补充 PATH 环境变量，默认 5 分钟超时

**窗口**
- `drag_window` / `set_window_background` / `minimize_to_tray`

### `src-tauri/src/chat.rs` — AI 对话

`send_chat` 命令：
- 接收 `ChatRequest { source, messages, max_tokens, temperature }`
- 兼容 OpenAI `/v1/chat/completions` 接口
- 通过 SSE 流式返回，emit 事件：
  - `chat-chunk` — 内容增量
  - `chat-chunk-reasoning` — 推理过程增量（如 DeepSeek-R1）
  - `chat-done` — 完成
  - `chat-error` — 错误
- `stop_chat` 通过 `ChatStopFlag` 中止

### `src-tauri/src/chat_sessions.rs` — 会话存储

JSON 文件存储，路径由 `get_chat_sessions_dir()` 管理。
命令：`list_chat_sessions`, `save_chat_session`, `update_chat_session_title`, `delete_chat_session`, `get_chat_session`

### `src-tauri/src/providers.rs` — Provider 管理

CRUD + 模型列表获取（`GET {base_url}/v1/models`）。
命令：`list_providers`, `save_provider`, `delete_provider`, `fetch_provider_models`, `set_active_model`, `get_provider`

### `src-tauri/src/command_runner.rs` — Shell 命令执行

独立模块，通过 `sh -c` 执行 shell 命令，流式输出结果。
- `run_command`: async 命令，接收 command/cwd/timeout_secs
- 用 `tokio::spawn` 异步读取 stdout/stderr
- 4 个 Tauri 事件：`command-stdout` / `command-stderr` / `command-done` / `command-error`
- 超时控制（默认 300 秒），自动补充常见 bin 目录到 PATH

### `pub fn run()` — Tauri 应用启动 (lib.rs)

- 注册所有 invoke 命令 handler
- 系统托盘（显示窗口 / 退出，Alt+Space）+ 全局快捷键
- 关闭窗口时隐藏到托盘而非退出

---

## 状态管理约定

1. **Svelte 5 Runes**：所有响应式状态使用 `$state`、`$derived`、`$effect`，不用 Svelte 4 store
2. **$state 对象**：跨页面共享用 `.svelte.ts` 的 `export const xxx = $state({...})`（如 `nm`、`toolbarState`）
3. **跨页面持久化**：`nmCache` 用普通 `Map` 在模块顶层声明，组件销毁不丢失
4. **组件内部状态**：直接 `let xxx = $state(...)`
5. **布局同步 store**：layout 用 `$effect()` 把 store 属性同步到局部 `$state`（解决跨导航响应式断裂）

---

## 开发约定

- **CSS**：全部用变量 `var(--xxx)`，变量定义在 `src/styles/global.css`，使用 scoped `<style>`
- **样式作用域**：组件内样式 scoped，全局用 `:global()` 覆盖
- **Tauri 调用**：`invoke<T>('command_name', { 参数 })`，注意 snake_case 参数名
- **事件监听**：`listen('event-name', cb)`，记得 `onDestroy` 清理
- **类型**：前端定义与后端 struct 对应的 TS interface
- **i18n**：UI 文案为中文，全角符号「」、emoji 视觉点缀
- ** materiais VISUAIS**：svg 内联图标，避免引入图标库

---

## 扩展功能时的起点

| 想做什么 | 从哪里开始 |
|---------|-----------|
| 新增 AI 可执行的 action | `chat/+page.svelte` 的 `handleActionClick` + `buildWorkspaceContext` |
| 新增 Tauri 命令 | `lib.rs` 写 `#[tauri::command]` + 在 `run()` 的 `generate_handler!` 注册 |
| 新增前端页面 | `src/routes/xxx/+page.svelte` + 在 `+layout.svelte` 的 `menuItems` 添加导航 |
| 新增跨页面共享状态 | `$lib/xxx.svelte.ts` 用 `export const xxx = $state({...})` |
| 修改 node_modules 扫描逻辑 | `lib.rs` 的 `scan_node_modules` + `src/lib/nm-store.svelte.ts` |
| 修改标题栏布局 | `+layout.svelte` 的 `.title-bar`、`.titlebar-loading`、`#titlebar-slot` |
| 修改 chat 工具栏 | `$lib/ChatToolbar.svelte` + `$lib/chat-state.svelte.ts` |
| 修改主题色彩 | `src/styles/global.css` 的 CSS 变量 |
| 修改代码块执行逻辑 | `MarkdownMessage.svelte` — `handleExecClick` |
| 修改命令执行后端 | `src-tauri/src/command_runner.rs` |
| 改对话协议 | `src-tauri/src/chat.rs` 的 `send_chat` |

## 可以使用的 skill

本项目中可配合使用的 Zed agent skills：

| Skill | 用途 | 路径 |
|-------|------|------|
| `svelte-code-writer` | Svelte 5 组件开发、文档查询、代码自动修复。创建或修改 `.svelte` 文件时必用 | `~/.agents/skills/svelte-code-writer/SKILL.md` |
| `project-guide` | 本项目架构指南（即本文档）。了解项目全貌、定位文件、扩展功能时使用 | `astra/.agents/skills/project-guide/SKILL.md` |
| `tauri-v2` | Tauri v2 跨平台开发，配置 `tauri.conf.json`、添加 Rust 命令、IPC 模式、权限配置 | `~/.agents/skills/tauri-v2/SKILL.md` |

**使用方式**：在对话中 @ 对应的 skill 名称即可调用，例如 `@svelte-code-writer 帮我创建这个组件`。

---

## 常用开发命令

```bash
pnpm dev              # Vite 开发服务器
pnpm tauri dev        # 完整 Tauri 应用（带 Rust 热重载）
pnpm build            # 前端构建
pnpm tauri build      # 打包 macOS app
pnpm check            # svelte-check 类型检查
pnpm check:watch      # 监听式类型检查
```

---

## 已知的 @ 提及和 Action 设计

@ 提及是核心互动入口，相关正则、状态、UI 集中在 `chat/+page.svelte`。

**@ 触发正则**（关键，修改需谨慎）：
- 检测：`/(?:^|\s)@([\w\u4e00-\u9fff.-]*)$/`
- 解析：`/@([\w\u4e00-\u9fff][\w\u4e00-\u9fff.-]*)/g`
- `\w` 不含中文，必须显式加 `\u4e00-\u9fff`

**Action 链接协议**：
- 格式：`[描述](action://command_type?param=value)`
- 前端拦截 `a[href^="action://"]`，通过 `new URL(href).hostname` 取命令类型
- 已实现：`open_project`、`run_command`
- 可扩展：`switch_workspace`、`create_project`（需弹窗确认）

**自动执行**：新消息中的 `open_project` action 链接由 `MarkdownMessage.svelte` 的 `autoExecuteActionLinks()` 自动触发，无需用户点击。通过 `isFresh` prop 控制（`chat/+page.svelte` 中新消息 `isFresh = true`，历史消息 `false`）。

**代码块执行**：`bash`/`sh`/`shell` 代码块自动显示执行按钮，`MarkdownMessage.svelte` 内部处理确认 → 执行 → 输出展示全流程。后端走 `command_runner.rs` 的 `run_command` 命令。

**注入到 AI 的系统提示由 `buildWorkspaceContext()` 动态生成**，会包含项目列表、action 协议说明和 shell 代码块执行说明。修改 action 行为或命令执行说明时同步更新这个函数的提示文案。
