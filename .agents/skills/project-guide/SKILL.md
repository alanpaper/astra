---
name: project-guide
description: 星野 (Astra) 项目的架构指南与开发约束。当在 astra 仓库中修改前端（React 19 页面、组件、zustand store）、Rust 后端（Tauri 命令、agent.rs 桥接层、会话存储）、新增页面/命令/全局状态、或排查 Agent 事件流 / 多会话问题时必须遵守。包含目录结构、Agent 并发约束、Tauri 命令清单与开发约定。
---

# 星野 (Astra) 项目架构指南

在 astra 仓库工作时遵循本指南。改代码前先读对应章节，了解文件职责边界与既有模式。

## 技术栈与数据流

- React 19 + zustand 5 + react-router 7 + Tauri 2 + Vite 6 + SCSS（旧 SvelteKit 已删除）
- 前端源码在 `src/src/`（`src/` 是前端包根，自带 package.json；Tauri 通过 `--dir src` 调用）
- 数据流：前端 → Tauri invoke → Rust 命令 → 文件系统 / SQLite
- Agent Runtime 在独立仓库 `astra-core`（多 crate workspace，crates/astra-* 16 个 crate），
  astra 通过 path 依赖引用：`agent-core = { package = "astra-agent", path = "../../astra-core" }`
  agent-core 拥有运行循环、权限判定与工具执行；astra 侧只做「宿主适配」
- 另按需引用 astra-core 子 crate：`astra-mcp`（MCP 客户端）/ `astra-acp`（ACP 适配）/
  `astra-process`（子进程）
- 主题：亮 / 暗 + 渐变背景（全局 CSS 变量），变量定义在 `src/src/styles/_variables.scss`
- 样式约定：SCSS + BEM 命名 + 只引用 var(--...) 主题变量；每页面一个同名 .scss

## 目录结构

```
astra/
├── src/                          # 前端包根（package.json / vite.config / tsconfig）
│   └── src/
│       ├── App.tsx               # react-router 路由表
│       ├── main.tsx
│       ├── pages/                # 页面（编排层）+ 页面级子组件
│       │   ├── WorkspacePage.tsx         # 工作空间页
│       │   ├── AgentPage.tsx             # ★ Agent 页（编排层 + EntryView 等内联子组件）
│       │   ├── AgentTabs.tsx/.scss       # ★ 多会话标签栏（S0）
│       │   ├── AgentHistoryPanel.tsx/.scss # ★ 历史侧栏即导航（全局平铺 + 时间分桶 + 项目徽标）
│       │   ├── chat/                     # 对话页子组件（Composer/MarkdownMessage/MentionPopup/...）
│       │   ├── models/ providers/ skills/ settings/ logs/ tools/ dev-mode/
│       ├── stores/               # zustand 全局 store
│       │   ├── agent.ts          # ★ Agent 会话全局状态（含多会话 S0；模块级事件监听）
│       │   ├── agent.test.mjs    # node:test 单测（transpile + vm 加载 agent.ts）
│       │   ├── workspace.ts / chatToolbar.ts / logs.ts / favorite.ts / nm.ts
│       ├── components/           # 跨页面组件（agent/ chat/ ui/Modal illustrations/ layout/）
│       ├── hooks/                # useMention / useSlash / useOutputScroll / ...
│       ├── features/             # home / models 页面子模块
│       └── styles/               # _variables.scss（主题变量）+ global.scss + ui.scss
└── src-tauri/
    ├── tauri.conf.json           # beforeDevCommand: pnpm --dir src dev; frontendDist: ../src/dist
    └── src/
        ├── lib.rs                # 命令注册、工作空间、项目、编辑器、模型
        ├── agent.rs              # ★ Agent Runtime 桥接层（agent-core → Tauri）+ checkpoint 命令
        ├── extensions.rs         # MCP/ACP 扩展管理（读写 ~/.astra/config.toml + 启停/删除）
        ├── mcp_config_tool.rs    # 宿主工具 mcp_config_read/write（让 agent 帮用户装/改 MCP）
        ├── memory.rs             # 记忆系统桥（MemoryHost 加密 sqlite → MemoryProvider）
        ├── market.rs             # 本地扩展市场状态（不读远程；安装引导回 Agent 对话）
        ├── agent_threads.rs      # Thread 持久化（SQLite + 错误上报）
        ├── agent_sessions.rs     # Agent 会话历史（JSON 文件）
        ├── session_store.rs      # 会话 JSON 共享辅助（chat/agent 共用）
        ├── user_skills.rs        # 应用自有技能库（app_data_dir/skills）
        ├── chat.rs / chat_sessions.rs # 简单流式对话（非 Agent）+ 历史
        ├── command_runner.rs / dev_mode.rs / downloader.rs
        ├── embedding.rs / indexer.rs / vector_store.rs   # RAG 子系统
        └── providers.rs          # Provider CRUD（providers.json）
```

## Agent 架构要点（改 agent.rs 或 agent.ts 前必读）

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
- **多会话（S0）已在前端落地但后端仍单活动 Run**：前端 UI 互斥（`runningSessionId()` 守卫
  canSend / handleSend），运行中的会话可切走驻留、事件持续路由，但发送必须等它结束
- 若要真正的并发多开：Thread 已按 session_id 隔离，卡点在 active 单槽与 threads 锁的持有粒度，
  需改为按 session 隔离的并发模型（SESSION_RUNTIME_SPEC P3）

### 多会话运行时（S0，设计冻结见 astra-core/docs/SESSION_RUNTIME_SPEC.md）

- **tab 即 AgentSession 视图**：`AgentTabs.tsx` 渲染 `openTabs`；激活会话字段平铺在 store，
  非激活会话快照驻留 `parked: Record<string, ParkedSession>`（swap 模型，切回零重建）
- **事件按 session_id 路由**：`agent-event` 监听器区分 flat / parked；parked 运行中的会话
  持续吸收事件，终端事件后进入 persisting 屏障等 `agent-run-settled`
- **tab 状态并入 agent store**（openTabs / activeTabId / isDraft，易失不持久化）——
  因 `agent.test.mjs` 的 vm mock 只认 4 个模块，agent.ts **不得新增 import**
- **保护规则**：running / settling / 待决策卡（`hasPendingDecision`）/ 输入非空的会话不可被
  LRU 驱逐或关闭 tab；`MAX_RETAINED_TABS = 8`，超限按 lastActive 清理（先落库再 reset）
- **草稿语义**：空草稿（未发首条消息）不驻留、不保存、其 tab 被目标顶替丢弃；`pushUser` /
  发送后 `isDraft=false` 转正
- **历史即导航**：`AgentHistoryPanel` 全局平铺（不按项目分组，项目只是会话属性）、时间分桶
  （今天/昨天/更早）、项目徽标、运行中/待处理/打开中标识；点条目 `activateSession` 换入
- **应用重启恢复**：`refreshAgentStatus` 发现未知运行中会话 → 建驻留快照并自动开 tab

### 权限桥与提问桥

- `PermissionHub`：tool_call_id → oneshot::Sender<Decision>；**批准只放行该次具体调用，不扩大范围**
- 放权模式 `agent_set_auto_all(true)`：Confirm 类直接 Allow；Deny 硬规则（未注册工具、提权命令、路径逃逸）不受影响
- 发送端被清理（取消/重置）时 receiver 收 Err → 自动 Deny，不挂死运行循环
- `QuestionHub`：question_id → oneshot；用户取消回传 None → 工具收到错误

### MCP/ACP 扩展

- 扩展配置在 `~/.astra/config.toml`（[mcp_servers.*] / ACP 段），由 `extensions.rs` 读写
- **装配时机**：每次 `agent_start_run` 前按当前启停表 `attach_extensions`（agent.rs）；
  但 **MCP 工具集合在建 Thread 时冻结**——新装/启停 MCP 后需新开会话才生效
- `required` 扩展启动失败阻断 Run，其余失败降级为提示；运行态经
  `extensions_runtime_status` 暴露（MarketPage 展示启停/可用徽标）
- **agent 可帮用户改配置**：宿主工具 `mcp_config_read`（Allow）/
  `mcp_config_write`（Confirm；upsert / set_enabled / remove，toml_edit 保留注释，
  写后 astra-config schema 校验失败自动回滚）。新装 MCP 的引导话术是
  「回 Agent 对话说一句」，MarketPage 不做远程拉取
- **净化 PATH 坑**：MCP server 的 `command` 必须用绝对路径（如
  `/opt/homebrew/bin/uvx`、fnm node 的绝对路径），否则 spawn 报 os error 2
- 新增宿主工具必须同步加进 agent.rs 的 ToolPolicy 白名单
  （allowed_readonly / confirm_tools），**未知工具默认 Deny**

### 记忆系统（memory.rs + astra-memory）

- 宿主实现 `MemoryHost`：加密 sqlite（`memory.key` 存密钥，XChaCha20-Poly1305），
  实现 astra-core 的 `MemoryProvider` trait，经 AgentState 注入
- 命令：`memory_status`（可用态）/ `memory_search`（语义检索）/ `memory_remember` /
  `memory_forget`；依赖 astra-memory 的嵌入能力
- **recall 注入**：agent_start_run 时检索相关记忆拼进 system prompt
  （REMEMBERED CONTEXT 段）——agent 静默使用，不向用户解说记忆的存在

### 会话模型

- session_id 由**前端生成**；Thread 按 session_id 缓存，创建时绑定 project_root 与模型 backend —— 切换项目仍需新会话；**模型/图片识别配置已解绑**（切换后 dest_changed 自动卸载缓存 Thread 并重建，当前会话即时生效，无需新开）；**MCP 工具集合仍冻结**（见上节，新会话生效）
- `agent_reset_session` 只 unload 内存；重启历史由 SQLite 负责；多会话下仅在**关闭 tab /
  LRU 驱逐 / 手动新对话**时调用（切走驻留不清后端上下文）
- 持久化失败记入 `PersistenceErrors`，经 `agent_status` 暴露；有未落盘后缀的 Thread 拒绝 unload
- 前端条目流另存 `agent_sessions.json`，恢复时原样回放；`session_files: HashSet` 防止删除后的迟到保存
- Checkpoint：`agent_checkpoint_list / create / restore`（agent.rs），前端 `CheckpointButton`

## 前端约定

- **页面编排层**：`pages/XxxPage.tsx` 只做数据加载与状态协调，子组件拆同目录或 `components/`
- **全局 Store**：`stores/xxx.ts`，zustand `create` + 选择器订阅（selector 返回原始值/引用，
  不要返回新建对象）；仅单页面用的 useState 留在页面组件
- **事件监听**：模块级单例（见 agent.ts `ensureAgentListeners`）——切页不丢事件；返回的
  unlisten 仅在注册失败回滚时调用
- **弹窗**：用 `components/ui/Modal.tsx`
- **样式**：先查 `styles/ui.scss` 共享类；页面样式 BEM + var(--...)；图标用 lucide-react
- **@ 提及正则**（改动需谨慎，`\w` 不含中文必须显式加 `\u4e00-\u9fff`）：
  检测 `/(?:^|\s)@([\w\u4e00-\u9fff.-]*)$/`，解析 `/@([\w\u4e00-\u9fff][\w\u4e00-\u9fff.-]*)/g`
- **action:// 协议**：`[描述](action://command_type?param=value)`，`autoExecuteActionLinks()` 自动执行
- **agent.ts 改动必须跑** `node --test src/src/stores/agent.test.mjs`（在 `src/` 下执行
  `node --test src/stores/agent.test.mjs`）；新增 store 能力先补测试

## 后端约定

- 新增命令：模块内写 `#[tauri::command]`，在 `lib.rs` 的 `run()` 中注册
- 文件头部用 `//!` 写职责边界说明（参考 agent.rs / user_skills.rs 的分层注释风格）
- chat.rs 是**全局单流**设计（单 `ChatStopFlag`，事件无会话 ID），不要在它上面叠加多会话需求——多会话走 agent.rs
- 会话历史用 `session_store.rs` 的共享辅助（ensure_json_file / read_json_vec），不要另写 JSON 读写
- vector_store.rs 的核心逻辑拆 `*_conn` 函数（接受 `&Connection`）便于单测
- **search_web 引擎可配置**：AppSettings.search_engine（bing | duckduckgo | searxng）
  → `agent_core::net_tools::SearchEngine::resolve` → `SearchTools::with_engine`；
  默认 Bing，仅配了 searxng endpoint 未选引擎时保持旧语义

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
agent_checkpoint_list / agent_checkpoint_create / agent_checkpoint_restore
extensions_runtime_status   // MCP/ACP 运行态（供 market 页启停/可用徽标）
market_status               // 本地扩展市场状态（不读远程）

// MCP/ACP 扩展管理（extensions.rs）
extensions_list / extensions_set_enabled / extensions_remove / extensions_update

// 记忆系统（memory.rs）
memory_status / memory_search / memory_remember / memory_forget

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
# 前端（前端包根是 src/）
pnpm --dir src dev         # Vite 开发服务器
pnpm --dir src typecheck   # tsc --noEmit
pnpm --dir src build       # 前端构建
pnpm tauri dev             # 完整 Tauri 应用（仓库根，带 Rust 热重载）
pnpm tauri build           # 打包 macOS app

# Agent store 单测（在 src/ 下）
cd src && node --test src/stores/agent.test.mjs
```

## 相关 skill

- **rust-agent-runtime** — 修改 agent.rs / agent_threads.rs 或 astra-core 时必须加载
- **build-agent-runtime** — 理解 agent runtime 设计参考
- **tauri-v2** — Tauri 命令、权限配置、IPC 细节
- **rust-best-practices** — Rust 代码规范
- 设计冻结文档：astra-core/docs/ `AGENT_WORKSPACE_SPEC.md` / `SESSION_RUNTIME_SPEC.md`
