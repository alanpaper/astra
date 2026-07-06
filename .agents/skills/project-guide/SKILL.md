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

核心两大功能页面：
1. **工作空间** (`/`) — 管理工作空间目录、扫描项目、创建/打开项目
2. **对话** (`/chat`) — 与 AI 模型对话，支持 @ 项目提及、action 链接触发命令

---

## 目录结构

```
astra/
├── src/                          # 前端源码
│   ├── lib/                      # 共享模块
│   │   ├── theme.svelte.ts       # 主题管理（亮/暗 + 渐变背景）
│   │   └── workspace.svelte.ts   # 全局工作空间状态（chat 和 workspace 页面共享）
│   ├── routes/                   # SvelteKit 路由
│   │   ├── +layout.svelte        # 根布局：侧边栏导航 + 可拖拽标题栏
│   │   ├── +layout.ts            # SPA 配置 (ssr = false)
│   │   ├── +page.svelte          # 工作空间页面 (/)
│   │   ├── chat/                 # 对话页面 (/chat)
│   │   │   ├── +page.svelte
│   │   │   └── MarkdownMessage.svelte  # markdown 渲染组件
│   │   ├── models/               # 本地模型管理 (/models, /models/[id])
│   │   ├── providers/            # API Provider 配置 (/providers, /providers/[id])
│   │   ├── settings/             # 设置页面 (/settings)
│   │   └── skills/               # Skills 管理 (/skills, /skills/[name])
│   ├── styles/
│   │   └── global.css            # 全局 CSS 变量 + 样式
│   └── app.html
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── main.rs               # 入口
│   │   ├── lib.rs                # 主库：工作空间、项目、编辑器、模型服务器、Skills
│   │   ├── chat.rs               # AI 对话（OpenAI 兼容接口，流式 SSE）
│   │   ├── chat_sessions.rs      # 对话会话持久化 (JSON 文件存储)
│   │   └── providers.rs          # API Provider CRUD + 模型列表获取
│   ├── capabilities/             # Tauri 权限配置
│   ├── tauri.conf.json           # Tauri 主配置
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
  loadFromSettings()  // 加载设置 + 扫描
  scanProjects(path)
  switchWorkspace(path)
  findProjectByName(name)
}
export const workspaceStore  // 单例
```

**重要**：chat 页面和 workspace 页面都应使用此 store 来保持数据同步。

### `src/routes/+layout.svelte` — 根布局

- 可拖拽顶部标题栏（透传 `#titlebar-slot` 给子页面 portal 工具栏）
- 左侧可折叠侧边栏（工作空间、对话、设置）
- 全局快捷键：`Cmd+,` 打开设置，`Cmd+W` 最小化到托盘
- 移动端汉堡菜单

### `src/routes/+page.svelte` — 工作空间页面

项目卡片列表、搜索、详情视图、新建项目。

核心函数：`loadAndScan`, `switchWorkspace`, `scanWorkspace`, `openProject`, `createNewProject`, `showDetail`, `openEditorForPath`

### `src/routes/chat/+page.svelte` — 对话页面（核心功能页）

这是项目最复杂的页面，包含以下子系统：

1. **来源切换**：Provider (API Key) 或 本地模型 (llama.cpp 服务器)
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
   - `handleActionClick` 拦截点击 → 调用 Tauri 命令（如 `open_in_editor`）
   - 三种状态：`action-executed`（执行中）、`action-done`（成功，pointer-events: none）、`action-error`（失败）
   - 防重复点击

### `src/routes/chat/MarkdownMessage.svelte` — Markdown 渲染

使用 `marked` + `highlight.js` 代码高亮 + 代码块复制按钮。action 链接通过此组件渲染为 `<a href="action://...">`。

### 其他路由

- `/settings` — 工作空间配置、默认编辑器、扫描深度
- `/providers` + `/providers/[id]` — API Provider CRUD（OpenAI 兼容）
- `/models` + `/models/[id]` — 本地模型配置（支持启动 llama.cpp 服务器）
- `/skills` + `/skills/[name]` — 浏览/删除系统中的 Skills

---

## 后端文件职责 (Rust)

### `src-tauri/src/lib.rs` — 主库

包含绝大多数 Tauri 命令：

**工作空间 / 项目**
- `scan_workspace(path)` — 递归扫描工作空间下的项目（检测 README.md）
- `get_project_detail(path)` — 项目详情（子项目、git remote、README 预览）
- `create_project(workspace_path, folder_name, project_name)` — 创建项目目录
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

### `pub fn run()` — Tauri 应用启动 (lib.rs)

- 注册所有 invoke 命令 handler
- 系统托盘（显示窗口 / 退出，Alt+Space）+ 全局快捷键
- 关闭窗口时隐藏到托盘而非退出

---

## 状态管理约定

1. **Svelte 5 Runes**：所有响应式状态使用 `$state`、`$derived`、`$effect`，不用 Svelte 4 store
2. **$state class 实例**：`workspaceStore` 是 class 实例，赋值给普通 `export const`，类成员用 `$state` 字段
3. **跨页面共享**：用 `$lib/workspace.svelte.ts`，而非 props 或 context
4. **组件内部状态**：直接 `let xxx = $state(...)`

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
| 新增 AI 可执行的 action | chat/+page.svelte 的 `handleActionClick` + `buildWorkspaceContext` + `tauri::command` |
| 新增 Tauri 命令 | `lib.rs` 写 `#[tauri::command]` + 在 `run()` 的 `generate_handler!` 注册 |
| 新增前端页面 | `src/routes/xxx/+page.svelte` + 在 `+layout.svelte` 的 `menuItems` 添加导航 |
| 新增跨页面共享状态 | `$lib/xxx.svelte.ts` 用 class + `$state` 字段 |
| 修改主题色彩 | `src/styles/global.css` 的 CSS 变量 |
| 改对话协议 | `src-tauri/src/chat.rs` 的 `send_chat` |

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
- 已实现：`open_project`
- 可扩展：`switch_workspace`、`create_project`（需弹窗确认）

**注入到 AI 的系统提示由 `buildWorkspaceContext()` 动态生成**，会包含项目列表和 action 协议说明。修改 action 行为时同步更新这个函数的提示文案。
