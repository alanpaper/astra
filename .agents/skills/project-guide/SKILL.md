---
name: project-guide
description: 星野 (Astra) 项目的架构指南与开发约束。当在 astra 仓库中修改前端（SvelteKit 2 / Svelte 5 页面、组件、store）、Rust 后端（Tauri 命令、agent.rs 桥接层、会话存储）、新增页面/命令/全局状态、或排查 Agent 事件流问题时必须遵守。包含目录结构、Agent 并发约束、Tauri 命令清单与开发约定。
---

# 星野 (Astra) 项目架构指南

在 astra 仓库工作时遵循本指南。改代码前先读对应章节，了解文件职责边界与既有模式。

## 技术栈与数据流

- SvelteKit 2 + Svelte 5 + Tauri 2 + Vite 6
- 数据流：前端 → Tauri invoke → Rust 命令 → 文件系统 / SQLite
- Agent Runtime 在独立仓库 `astra-core`（多 crate workspace），astra 通过 path 依赖引用：
  `agent-core = { package = "astra-agent", path = "../../astra-core" }`
  agent-core 拥有运行循环、权限判定与工具执行；astra 侧只做「宿主适配」
- 主题：亮 / 暗 + 渐变背景（全局 CSS 变量），CSS 变量定义在 `src/styles/global.css`（90+ 变量）

## 目录结构

```
astra/
├── src/                          # 前端
│   ├── lib/
│   │   ├── ui/Modal.svelte       # 通用弹窗（Esc 关闭/confirm 变体/防误触）
│   │   ├── home/                 # 工作空间页子组件（ProjectCard/ProjectDetail/...）
│   │   ├── ChatToolbar.svelte    # 对话页工具栏（渲染在标题栏）
│   │   ├── chat-state.svelte.ts  # 对话页工具栏状态
│   │   ├── favorite-store.svelte.ts / logs-store.svelte.ts / nm-store.svelte.ts
│   │   ├── theme.svelte.ts / workspace.svelte.ts / format.ts
│   ├── routes/
│   │   ├── +layout.svelte        # 侧边栏 + 标题栏（titlebar-slot）
│   │   ├── +page.svelte          # 工作空间页（编排层）
│   │   ├── chat/                 # 对话页（编排层 + 7 子组件）
│   │   │   ├── ChatMessageView / Composer / HistoryDrawer
│   │   │   ├── MarkdownMessage / MentionPopup / SettingsSheet / Welcome
│   │   │   ├── action-links.ts         # action:// 链接解析与执行
│   │   │   ├── types.ts / workspace-context.ts
│   │   ├── models/               # 本地模型管理（编排层 + ModelCard/FormModal/ScanPanel）
│   │   ├── providers/            # Provider CRUD（列表 + [id] 详情）
│   │   ├── skills/               # 系统技能浏览（~/.agents/skills 全局 Git 技能）
│   │   ├── settings/ logs/ dev-mode/[encoded]/
│   │   └── tools/                # 工具箱（downloader / nm）
│   └── styles/                   # global.css（变量）+ ui.css（共享类）
└── src-tauri/src/
    ├── lib.rs                    # 命令注册、工作空间、项目、编辑器、模型
    ├── agent.rs                  # ★ Agent Runtime 桥接层（agent-core → Tauri）
    ├── agent_threads.rs          # Thread 持久化（SQLite + 错误上报）
    ├── agent_sessions.rs         # Agent 会话历史（JSON 文件）
    ├── session_store.rs          # 会话 JSON 共享辅助（chat/agent 共用）
    ├── user_skills.rs            # 应用自有技能库（app_data_dir/skills）
    ├── chat.rs / chat_sessions.rs # 简单流式对话（非 Agent）+ 历史
    ├── command_runner.rs / dev_mode.rs / downloader.rs
    ├── embedding.rs / indexer.rs / vector_store.rs   # RAG 子系统
    └── providers.rs              # Provider CRUD（providers.json）
```

## Agent 架构要点（改 agent.rs 前必读）

### 事件流

```
前端 invoke agent_start_run
  → agent.rs 组装 ToolRegistry / ModelBackend / PermissionBridge
  → agent-core 运行循环
  → Envelope 事件经 app.emit("agent-event", ...) 推给前端
```

- 前端只消费 Envelope（type 标签 + snake_case 载荷），不依赖 agent-core 具体类型
- 所有 Envelope 追加写入 `app_data_dir/agent_events.log`（JSONL，带 logged_at），供 /logs 页排查
- ask_user 走单独的 `agent-question` 事件（payload 含 session_id / question_id / options）

### 并发模型（单活动 Run）★

- `AgentState.active` 是单槽 `Option<(session_id, RunHandle)>` —— **同一时刻全局只允许一个活动 Run**
- `agent_start_run` 用 `threads.try_lock_owned()` 独占准入，运行中再调用返回
  "Agent is running, draining or changing sessions"
- 若要支持多开 agent 对话：Thread 已按 session_id 隔离，卡点在 active 单槽与 threads 锁的持有粒度，需改为按 session 隔离的并发模型

### 权限桥与提问桥

- `PermissionHub`：tool_call_id → oneshot::Sender<Decision>；**批准只放行该次具体调用，不扩大范围**
- 放权模式 `agent_set_auto_all(true)`：Confirm 类直接 Allow；Deny 硬规则（未注册工具、提权命令、路径逃逸）不受影响
- 发送端被清理（取消/重置）时 receiver 收 Err → 自动 Deny，不挂死运行循环
- `QuestionHub`：question_id → oneshot；用户取消回传 None → 工具收到错误

### 会话模型

- session_id 由**前端生成**；Thread 按 session_id 缓存，创建时绑定 project_root 与模型 backend —— **切换项目 / 模型来源后必须新开会话**
- `agent_reset_session` 只 unload 内存；重启历史由 SQLite 负责
- 持久化失败记入 `PersistenceErrors`，经 `agent_status` 暴露；有未落盘后缀的 Thread 拒绝 unload
- 前端条目流另存 `agent_sessions.json`，恢复时原样回放；`session_files: HashSet` 防止删除后的迟到保存

## 前端约定

- **编排层模式**：`+page.svelte` 只做数据加载与状态协调，子组件拆到同目录或 `lib/`
- **全局 Store**：`src/lib/xxx-store.svelte.ts`，`$state` + `class XxxStore` 单例导出；仅单页面用的状态留在 `+page.svelte`
- **事件监听**：`onMount` 中 `listen('event-name', cb)`，返回的 unlisten 在 `onDestroy` 清理
- **弹窗**：用 `lib/ui/Modal.svelte` + `ui.css`，不要各页面自实现。表单类传 `closeOnOverlay={false}`，确认类传 `variant="confirm"`
- **样式**：先查 `ui.css`（btn/modal/form/empty-state/badge 等共享类），再考虑 scoped 样式
- **@ 提及正则**（改动需谨慎，`\w` 不含中文必须显式加 `\u4e00-\u9fff`）：
  检测 `/(?:^|\s)@([\w\u4e00-\u9fff.-]*)$/`，解析 `/@([\w\u4e00-\u9fff][\w\u4e00-\u9fff.-]*)/g`
- **action:// 协议**：`[描述](action://command_type?param=value)`，已实现 `open_project` / `run_command`；新消息由 `autoExecuteActionLinks()` 自动执行（`isFresh` 控制）

> 注意：前端目前**没有 Agent 页面**（后端 agent.rs 已就绪），/chat 仍走旧 `chat.rs`（全局单流、事件无会话 ID）。新建 Agent 前端的对接点是 `agent_start_run` + `agent-event` 事件流。

## 后端约定

- 新增命令：模块内写 `#[tauri::command]`，在 `lib.rs` 的 `run()` 中注册
- 文件头部用 `//!` 写职责边界说明（参考 agent.rs / user_skills.rs 的分层注释风格）
- chat.rs 是**全局单流**设计（单 `ChatStopFlag`，事件无会话 ID），不要在它上面叠加多会话需求——多会话走 agent.rs
- 会话历史用 `session_store.rs` 的共享辅助（ensure_json_file / read_json_vec），不要另写 JSON 读写
- vector_store.rs 的核心逻辑拆 `*_conn` 函数（接受 `&Connection`）便于单测

## Tauri 命令清单

```rs
// 工作空间 & 项目
list_workspaces / switch_workspace / add_workspace / remove_workspace / get_default_workspace
scan_workspace / create_project / get_project_size
scan_node_modules / delete_node_modules / cancel_nm_scan / cancel_nm_delete
open_in_editor / open_project / resolve_test_path / open_file_picker / open_folder_picker

// 模型
list_local_models / run_llama_server

// 简单对话（/chat 页）
send_chat / stop_chat
list_chat_sessions / save_chat_session / get_chat_session / delete_chat_session

// ★ Agent
agent_start_run        // 启动一轮 Run，事件经 agent-event 流出，返回 session_id
agent_cancel           // 取消当前 Run（丢弃挂起权限/问题请求）
agent_permission_decide(tool_call_id, decision)
agent_question_answer(question_id, answer)   // None = 取消
agent_set_auto_all(enabled)
agent_reset_session(session_id)
agent_status           // running / session_id / persistence_errors
agent_recent_events(limit)

// Agent 会话
list_agent_sessions / save_agent_session / delete_agent_session

// 用户技能库（应用自有，app_data_dir/skills）
list_user_skills / create_user_skill / import_user_skill / sync_global_skills
list_user_skill_files / read_user_skill_file / write_user_skill_file
delete_user_skill_file / delete_user_skill

// 系统技能（~/.agents/skills）
scan_skills / delete_skill / read_skill_doc / update_skill / update_all_skills

// 嵌入 & 向量检索
embed_text / test_embedding_source / store_chunks / delete_project_index
search_similar / get_index_stats
```

## 常用开发命令

```bash
export PATH="/Users/hanbiao/.local/share/fnm/node-versions/v22.14.0/installation/bin:$PATH"
pnpm dev              # Vite 开发服务器
pnpm tauri dev        # 完整 Tauri 应用（带 Rust 热重载）
pnpm build            # 前端构建
pnpm tauri build      # 打包 macOS app
pnpm check            # svelte-check 类型检查
```

## 相关 skill

- **rust-agent-runtime** — 修改 agent.rs / agent_threads.rs 或 astra-core 时必须加载
- **build-agent-runtime** — 理解 agent runtime 设计参考
- **tauri-v2** — Tauri 命令、权限配置、IPC 细节
- **svelte-core-bestpractices** — 编写 Svelte 5 组件时加载
- **rust-best-practices** — Rust 代码规范
