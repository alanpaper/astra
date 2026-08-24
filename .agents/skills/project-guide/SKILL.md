---
name: project-guide
description: 星野 (Astra) 项目的完整架构指南。说明项目目录结构、前后端文件职责、Tauri 命令清单、状态管理方式和开发约定。当需要了解项目全貌、定位文件、扩展功能或修改架构时使用此 skill。
---

# 星野 (Astra) 项目架构指南

## 项目概览

- **技术栈**: SvelteKit 2 + Svelte 5 + Tauri 2 + Vite 6
- **数据流**: 前端 → Tauri invoke → Rust 命令 → 文件系统 / SQLite
- **主题**: 亮 / 暗 + 渐变背景绘画风格（全局 CSS 变量控制）
- **特色**: @ 项目提及 → AI 识别 → 自动打开项目 / 文件

项目已通过渐进式重构，将原来的巨型单文件页面（chat 2444 行、workspace 2222 行、models 2064 行）拆分为 **编排层 + 组件** 的架构模式，显著提升可维护性。

---

## 目录结构

```
astra/
├── src/                          # 前端源码
│   ├── lib/                      # 共享模块
│   │   ├── ui/                   # 共享 UI 组件
│   │   │   └── Modal.svelte      # 通用弹窗（Esc 关闭/confirm 变体/防误触）
│   │   ├── home/                 # 工作空间页面的子组件
│   │   │   ├── CreateProjectModal.svelte   # 新建项目弹窗
│   │   │   ├── NodeModulesPanel.svelte     # node_modules 扫描/清理面板
│   │   │   ├── ProjectCard.svelte          # 项目卡片
│   │   │   ├── ProjectDetail.svelte        # 项目详情（含 node_modules 面板）
│   │   │   ├── folder-utils.ts             # 目录扫描辅助函数
│   │   │   └── types.ts                    # 工作空间共用类型
│   │   ├── format.ts             # 全局格式化函数（字节/时间/速度）
│   │   ├── theme.svelte.ts       # 主题管理（亮/暗 + 渐变背景）
│   │   ├── workspace.svelte.ts   # 全局工作空间状态
│   │   ├── nm-store.svelte.ts    # node_modules 扫描/清理全局状态
│   │   ├── chat-state.svelte.ts  # 对话页工具栏共享状态
│   │   ├── favorite-store.svelte.ts  # 收藏开发模式状态
│   │   ├── logs-store.svelte.ts      # 全局日志存储
│   │   └── ChatToolbar.svelte    # 对话页工具栏组件（在标题栏渲染）
│   ├── routes/                   # SvelteKit 路由
│   │   ├── +layout.svelte        # 根布局：侧边栏 + 可拖拽标题栏
│   │   ├── +layout.ts            # SPA 配置 (ssr = false)
│   │   ├── +page.svelte          # 工作空间页面 (/) — 编排层
│   │   ├── chat/                 # 对话页面 (/chat) — 编排层 + 子组件
│   │   │   ├── +page.svelte
│   │   │   ├── ChatMessageView.svelte  # 消息列表渲染
│   │   │   ├── Composer.svelte         # 输入框 + @ 提及
│   │   │   ├── HistoryDrawer.svelte    # 右侧会话历史抽屉
│   │   │   ├── MarkdownMessage.svelte  # Markdown 渲染 + action 执行
│   │   │   ├── MentionPopup.svelte     # @ 项目提及弹出框
│   │   │   ├── SettingsSheet.svelte    # 设置面板
│   │   │   └── Welcome.svelte          # 空状态欢迎页
│   │   ├── models/               # 本地模型管理 — 编排层 + 子组件
│   │   │   ├── +page.svelte
│   │   │   ├── ModelCard.svelte
│   │   │   ├── ModelFormModal.svelte
│   │   │   ├── ScanPanel.svelte
│   │   │   └── types.ts
│   │   ├── providers/            # API Provider 配置
│   │   │   ├── +page.svelte      # 列表页 —— 使用全局 Modal
│   │   │   └── [id]/+page.svelte # 详情页 —— 使用全局 Modal
│   │   ├── skills/               # Skills 管理
│   │   │   ├── +page.svelte
│   │   │   └── [name]/+page.svelte
│   │   ├── settings/             # 设置页面
│   │   │   └── +page.svelte
│   │   ├── logs/                 # 日志查看页面
│   │   │   └── +page.svelte
│   │   ├── dev-mode/[encoded]/   # 开发模式入口页
│   │   │   └── +page.svelte
│   │   └── tools/                # 工具箱
│   │       ├── +page.svelte
│   │       ├── downloader/+page.svelte   # 文件下载器 —— 使用全局 Modal
│   │       └── nm/+page.svelte          # node_modules 清理工具
│   ├── styles/
│   │   ├── global.css            # 全局 CSS 变量 + spin keyframes
│   │   └── ui.css                # 共享 UI 样式（Modal、Empty State 等）
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

### `src/lib/format.ts` — 全局共享格式化工具

提供时间、文件大小、下载速度等格式化函数，避免各页面重复实现。

**导出函数**：
- `formatBytes(bytes)` — 文件大小（2 位小数），用于项目源码大小、node_modules 大小等
- `formatSize(bytes)` — 下载尺寸显示（0 → '—'）
- `formatSpeed(bytesPerSec)` — 下载速度（0 → 空字符串）
- `formatTimestampSec(ts)` — 秒级时间戳 → 'YYYY-MM-DD HH:mm'
- `formatClockSec(ts)` — 秒级时间戳 → 'HH:mm:ss'
- `formatClockMs(ts)` — 毫秒时间戳 → 'HH:mm:ss'
- `formatRelativeSec(ts)` — 秒级时间戳 → 相对时间（刚刚 / N 分钟前 / N 小时前 / M月D日 HH:mm）
- `formatDayTimeSec(ts)` — 秒级时间戳 → 今天显示 'HH:mm'，否则 'M/D HH:mm'

### `src/styles/ui.css` — 共享 UI 样式

收集项目中跨页面重复的 CSS 模式，一次定义、多处复用。

**使用方式**：组件通过 `class="modal-overlay modal-content ..."` 直接应用，无需 scoped 样式。

**包含的样式类**：
- `.modal-overlay` / `.modal-content` — 弹窗层基础样式
- `.modal-title` / `.modal-body` / `.modal-footer` — 弹窗结构
- `.btn` / `.btn-primary` / `.btn-danger` / `.btn-ghost` — 按钮变体
- `.empty-state` — 空状态提示
- `.form-group` / `.form-label` / `.form-input` / `.form-select` — 表单元素
- `.badge` — 标签样式

### `src/lib/ui/Modal.svelte` — 通用弹窗组件

统一弹窗交互模式，提供一致的用户体验。

**Props**：
- `title` — 弹窗标题
- `onClose` — 关闭回调
- `closeDisabled` — 提交中时禁用关闭
- `closeOnOverlay` — 点击遮罩是否关闭（表单弹窗建议 `false` 防误触丢输入）
- `variant` — `"default"` | `"confirm"`（确认弹窗更紧凑，宽度 420px）
- `children` — 弹窗内容（Snipet）

**交互特性**：
- 支持 Escape 键关闭
- Spring 动画 `cubic-bezier(0.34, 1.56, 0.64, 1)`
- 表单弹窗传 `closeOnOverlay={false}` 防误触

**使用约定**：
- 表单类弹窗：`<Modal closeOnOverlay={false} title="..." onClose={...}>`
- 确认类弹窗：`<Modal variant="confirm" title="确认删除？" onClose={...}>`

### `src/lib/workspace.svelte.ts` — 全局工作空间状态

```ts
class WorkspaceStore {
  projects: Project[];       // 项目列表
  workspaces: Workspace[];   // 工作空间列表
  activePath: string;        // 当前激活工作空间路径
  activeName: string;        // 当前激活工作空间名称
  editor: string;            // 默认编辑器路径
  loading: boolean;          // 加载状态
}
const workspaceStore = new WorkspaceStore();
```

### `src/lib/nm-store.svelte.ts` — node_modules 全局状态

```ts
const nm = $state({
  scanning: boolean,
  cleaning: boolean,
  error: string | null,
  progress: string,
  activePath: string | null,
  cancelled: boolean
});
const nmCache: Map<string, NmEntry[]> = new Map(); // 路径 → 扫描结果缓存
```

### `src/lib/chat-state.svelte.ts` — 对话页工具栏状态

```ts
const toolbarState = $state({
  providerId: string | null,
  modelName: string | null,
  sourceType: 'provider' | 'model',
  showSettings: boolean
});
```

### `src/lib/favorite-store.svelte.ts` — 收藏开发模式状态

提供收藏的开发模式状态，用于全局快捷入口。

```ts
class FavoriteStore {
  favorite: DevModeFavorite | null;  // 当前收藏项（path + name）
  loading: boolean;
}
```

### `src/lib/logs-store.svelte.ts` — 全局日志存储

收集应用各处的日志记录（开发服务器输出、错误等）。

```ts
interface LogEntry {
  timestamp: number;
  level: 'info' | 'error' | 'warn';
  source: string;  // 例如 'dev-server', 'install', 'app'
  message: string;
}

class LogsStore {
  entries: LogEntry[];
  maxEntries: number;  // 默认 500
}
```

### `src/routes/+layout.svelte` — 根布局

侧边栏导航 + 可拖拽标题栏。

核心布局：
- 左侧固定宽度侧边栏（导航图标 + 底部设置）
- 顶部自定义标题栏（macOS 窗口控制按钮 + 标题）
- `titlebar-slot` 允许子页面在标题栏渲染内容（如 chat 页的工具栏）

### `src/routes/+page.svelte` — 工作空间页面（编排层）

项目卡片列表、搜索、详情视图、新建项目。

**当前架构**：700 行编排层 + `lib/home/` 下 4 个组件（ProjectCard、ProjectDetail、CreateProjectModal、NodeModulesPanel）。

**核心职责**：
- 加载工作空间列表、切换工作空间
- 扫描项目、渲染项目卡片
- 协调详情视图和新建项目弹窗

**node_modules 清理**：由 `NodeModulesPanel.svelte` 组件处理，通过 `nmCache` 缓存扫描结果。

### `src/routes/chat/+page.svelte` — 对话页面（编排层）

这是项目最复杂的页面，已从 2444 行单文件重构为 700 行编排层 + 8 个子组件：

| 组件 | 职责 |
|------|------|
| `ChatMessageView.svelte` | 消息列表渲染 |
| `Composer.svelte` | 输入框 + 发送逻辑 |
| `HistoryDrawer.svelte` | 右侧历史会话抽屉 |
| `MarkdownMessage.svelte` | Markdown 渲染 + action 执行 |
| `MentionPopup.svelte` | @ 项目提及弹出框 |
| `SettingsSheet.svelte` | 模型/Provider 设置面板 |
| `Welcome.svelte` | 空状态欢迎页 |
| `ChatToolbar.svelte` (lib) | 标题栏工具栏 |

**子系统**：
1. **来源切换**：通过 `chat-state.svelte.ts` 的 `toolbarState.sourceType` 控制 Provider / 本地模型
2. **流式对话**：监听 Tauri 事件 `chat-chunk` / `chat-chunk-reasoning` / `chat-done` / `chat-error`
3. **会话历史**：`HistoryDrawer` 右侧抽屉，搜索过滤
4. **@ 项目提及**：`MentionPopup` 处理触发 + 选择 + 插入逻辑，正则检测 `(?:^|\s)@([\w\u4e00-\u9fff.-]*)$`
5. **Workspace 联动**：`buildWorkspaceContext()` — 增强系统提示，注入项目列表和 action 协议
6. **Action 链接触发命令**：AI 回复中 `[🚀 打开](action://open_project?path=xxx)` 自动执行或点击执行
7. **代码块一键执行**：`MarkdownMessage` 处理 bash/sh/shell 代码块的执行确认 + 输出展示

### `src/routes/chat/MarkdownMessage.svelte` — Markdown 渲染 + 命令执行

使用 `marked` + `highlight.js` 代码高亮 + 代码块复制按钮。

**核心职责**：
- 渲染后自动扫描并执行 `action://open_project` 链接（仅新消息）
- 识别 `bash`/`sh`/`shell` 代码块，渲染 **▶ 执行** 按钮
- 内置确认弹窗（显示完整命令内容）
- 通过 Tauri 事件接收实时执行结果
- 防重复执行标记

### `src/routes/models/+page.svelte` — 本地模型管理（编排层）

已从 2064 行单文件重构为 448 行编排层 + 3 个组件：

| 组件 | 职责 |
|------|------|
| `ModelCard.svelte` | 模型卡片展示 |
| `ModelFormModal.svelte` | 新建/编辑模型表单弹窗（合并了 add/edit 逻辑） |
| `ScanPanel.svelte` | 扫描本地模型面板 |

使用全局 `Modal` 组件 + `ui.css` 样式。

### 其他路由

- `/settings` — 工作空间配置、默认编辑器、扫描深度
- `/providers` + `/providers/[id]` — API Provider CRUD（使用全局 Modal）
- `/skills` + `/skills/[name]` — 浏览/删除系统中的 Skills
- `/logs` — 日志查看页面（显示 `logs` store 中的条目）
- `/dev-mode/[encoded]` — 开发模式入口页（从 URL 参数解码项目信息并启动开发服务器）
- `/tools/downloader` — 文件下载器（使用全局 Modal）
- `/tools/nm` — node_modules 清理工具

---

## 后端文件职责 (Rust)

### `src-tauri/src/lib.rs` — 主库

暴露所有 Tauri 命令：

```rs
// 工作空间
#[tauri::command] fn list_workspaces() -> Vec<Workspace>
#[tauri::command] fn switch_workspace(path: String) -> Result<Workspace>
#[tauri::command] fn add_workspace(path: String) -> Result<Workspace>
#[tauri::command] fn remove_workspace(path: String) -> Result<()>
#[tauri::command] fn get_default_workspace() -> Option<Workspace>

// 项目
#[tauri::command] fn scan_workspace(path: String, depth: u32, include_dotfiles: bool) -> Result<Vec<Project>>
#[tauri::command] fn create_project(name: String, path: String) -> Result<Project>
#[tauri::command] fn get_project_size(path: String) -> u64

// node_modules 清理
#[tauri::command] fn scan_node_modules(path: String) -> Result<Vec<NmEntry>>  // path 为项目根目录
#[tauri::command] fn delete_node_modules(paths: Vec<String>) -> Result<u64>   // 返回释放字节数
#[tauri::command] fn cancel_nm_scan()
#[tauri::command] fn cancel_nm_delete()

// 编辑器 & 文件操作
#[tauri::command] fn open_in_editor(path: String, editor: String) -> Result<()>
#[tauri::command] fn open_project(path: String, editor: String) -> Result<()>
#[tauri::command] fn resolve_test_path(base: String, pattern: String) -> Result<String>
#[tauri::command] fn open_file_picker(title: String, filters: Vec<Filter>) -> Option<String>
#[tauri::command] fn open_folder_picker(title: String) -> Option<String>

// 模型
#[tauri::command] fn list_local_models(path: String) -> Vec<ModelInfo>.   // 扫描 GGUF 模型文件
#[tauri::command] fn run_llama_server(model: ModelConfig) -> Result<u16>   // 启动 llama.cpp 服务器

// Skills
#[tauri::command] fn scan_skills() -> Vec<Skill>
#[tauri::command] fn delete_skill(name: String) -> Result<()>
```

### `src-tauri/src/chat.rs` — AI 对话

管理 OpenAI 兼容 API 的流式对话：

- `build_chat_request()` — 构建请求体（含 system prompt + 历史消息）
- `stream_chat()` — 建立 SSE 连接，通过 Tauri 事件 emit chunk
- 事件：`chat-chunk`, `chat-chunk-reasoning`, `chat-done`, `chat-error`

### `src-tauri/src/chat_sessions.rs` — 会话存储

SQLite 持久化对话历史，JSON 存消息列表。

### `src-tauri/src/providers.rs` — Provider 管理

CRUD 操作，存储于 `providers.json`。

### `src-tauri/src/command_runner.rs` — Shell 命令执行

- `run_command(cmd, cwd, env)` — 执行 shell 命令
- 通过事件 `command-stdout`, `command-stderr`, `command-done`, `command-error` 推送结果
- 支持实时输出流

### `pub fn run()` — Tauri 应用启动 (lib.rs)

注册所有命令，初始化 SQLite 数据库。

---

## 状态管理约定

- **全局 Store**：`*.svelte.ts` 文件使用 `$state` 定义响应式状态
- **Store 导出**：导出 `const xxx = new XxxStore()` 单例
- **组件状态**：局部状态用 `$state` 在组件内部定义
- **事件监听**：在 `onMount` 中 `listen('event-name', callback)`，返回 `unlisten`
- **共享状态原则**：跨页面/组件共享的状态提升到 store；仅单页面内使用的状态留在 `+page.svelte`

---

## 开发约定

### 组件拆分原则

- **编排层**：`+page.svelte` 负责数据加载、状态协调、子组件组合
- **子组件**：按职责拆分到同目录或 `lib/` 下，接收 props、emit events
- **共享组件**：跨页面复用的组件放在 `src/lib/ui/`，配合 `ui.css` 共享样式

### CSS 模式

- **全局变量**：`src/styles/global.css` 定义 90+ 个 CSS 变量（颜色、间距、圆角、阴影等）
- **共享类**：`src/styles/ui.css` 提供按钮、表单、弹窗、空状态等可复用样式类
- **组件样式**：尽量复用 `ui.css` 类，避免 scoped 重复代码

### Modal 使用约定

- 表单类弹窗：`closeOnOverlay={false}` 防误触
- 确认类弹窗：`variant="confirm"` 更紧凑
- 所有弹窗默认支持 Escape 关闭
- 使用全局 Modal 而非各页面自己实现弹窗样式

---

## 扩展功能时的起点

1. **新增路由页面**：在 `src/routes/` 下创建目录，复制现有页面结构
2. **新增 Tauri 命令**：在 `src-tauri/src/lib.rs` 添加 `#[tauri::command]` 函数，然后在 `run()` 中注册
3. **新增全局状态**：在 `src/lib/` 创建 `xxx-store.svelte.ts`，导出单例
4. **新增样式**：先查 `ui.css` 是否有可复用类；无则添加到 `ui.css` 或用 scoped 样式
5. **新增组件**：先复用 `lib/ui/Modal` + `ui.css`；需共享时放 `lib/ui/`；仅单页面用放同目录

---

## 可以使用的 skill

- **svelte-code-writer** — 创建/编辑 Svelte 组件时使用，确保符合 Svelte 5 最佳实践
- **tauri-v2** — Tauri 命令、权限配置、IPC 调用时使用
- **card-converter** — 如果项目涉及 CASP card 转换时使用
- **shadcn** — 如果需要添加 shadcn/ui 组件时使用
- **vercel-react-best-practices** — React/Next.js 最佳实践参考（本项目是 Svelte，仅供参考模式）

---

## 常用开发命令

```bash
export PATH="/Users/hanbiao/.local/share/fnm/node-versions/v22.14.0/installation/bin:$PATH"
pnpm dev              # Vite 开发服务器
pnpm tauri dev        # 完整 Tauri 应用（带 Rust 热重载）
pnpm build            # 前端构建
pnpm tauri build      # 打包 macOS app
pnpm check            # svelte-check 类型检查
pnpm check:watch      # 监听式类型检查
```

---

## 已知的 @ 提及和 Action 设计

@ 提及是核心互动入口，**弹出框 UI** 在 `MentionPopup.svelte` 组件，**触发逻辑**由 `Composer.svelte` 中的输入检测控制。

**@ 触发正则**（关键，修改需谨慎）：
- 检测：`/(?:^|\s)@([\w\u4e00-\u9fff.-]*)$/`
- 解析：`/@([\w\u4e00-\u9fff][\w\u4e00-\u9fff.-]*)/g`
- `\w` 不含中文，必须显式加 `\u4e00-\u9fff`

**Action 链接协议**：
- 格式：`[描述](action://command_type?param=value)`
- 前端拦截 `a[href^="action://"]`，通过 `new URL(href).hostname` 取命令类型
- 已实现：`open_project`、`run_command`
- 可扩展：`switch_workspace`、`create_project`（需弹窗确认）

**自动执行**：新消息中的 `open_project` action 链接由 `MarkdownMessage.svelte` 的 `autoExecuteActionLinks()` 自动触发，无需用户点击。通过 `isFresh` prop 控制（chat 页新消息 `isFresh = true`，历史消息 `false`）。

**代码块执行**：`bash`/`sh`/`shell` 代码块自动显示执行按钮，`MarkdownMessage.svelte` 内部处理确认 → 执行 → 输出展示全流程。后端走 `command_runner.rs` 的 `run_command` 命令。