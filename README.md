# 星野 (Hoshino)

一款基于 Tauri v2 + SvelteKit 构建的现代化桌面项目管理工具。

## 功能特性

### 🗂️ 智能项目扫描

- 自动扫描工作空间下的所有项目目录
- 支持自定义扫描深度（默认 3 层）
- 基于 `.git` 目录检测，智能跳过无需遍历的目录


### 🎨 主题系统

- 完整的亮色 / 暗色主题支持
- 三种模式选择：
  - ☀️ 浅色模式
  - 🌙 深色模式
  - 🖥️ 跟随系统
- 基于 CSS 变量的语义化主题系统
- 防闪烁内联脚本，切换无白屏

### 📐 VS Code 风格侧边栏

- 折叠时缩小到 48px 宽（仅图标）
- 点击头部 logo 切换展开/收起
- 默认收起状态，节省屏幕空间

### 🔧 系统托盘集成

- 关闭按钮最小化到托盘，而非退出
- 托盘菜单快速访问：
  - 显示/隐藏窗口
  - 打开设置
  - 退出应用

### ⌨️ 全局快捷键

| 快捷键 | 功能 |
|--------|------|
| `⌥ + Space` (Option+Space) | 全局唤醒/隐藏窗口 |
| `⌘ + ,` (Cmd+,) | 打开设置页面 |
| `⌘ + W` (Cmd+W) | 最小化到托盘 |

## 技术栈

- **前端**: SvelteKit + TypeScript
- **后端**: Tauri v2 + Rust
- **构建工具**: Vite + pnpm

## 开发环境

### 前置要求

- [Node.js](https://nodejs.org/) (推荐 v18+)
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/)
- [VS Code](https://code.visualstudio.com/)

### 推荐 VS Code 扩展

- [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode)
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

### 安装依赖

```bash
pnpm install
```

### 开发模式

```bash
pnpm tauri dev
```

### 构建发布

```bash
pnpm tauri build
```

## 项目结构

```
hoshino/
├── src/                    # SvelteKit 前端源码
│   ├── lib/               # 共享组件和工具
│   │   └── theme.svelte.ts # 主题状态管理
│   ├── routes/            # 页面路由
│   │   ├── workspace/     # 工作空间页面
│   │   ├── settings/      # 设置页面
│   │   └── skills/        # 技能页面
│   └── app.html           # HTML 模板（含防闪烁脚本）
├── src-tauri/             # Tauri/Rust 后端
│   ├── src/               # Rust 源码
│   │   └── lib.rs         # 主要后端逻辑
│   └── tauri.conf.json    # Tauri 配置
└── package.json           # 前端依赖
```

## 配置说明

### 扫描深度

在设置页面可调整项目扫描的递归深度（1-10），默认为 3。深度越大，扫描越深入，但耗时也会增加。

### 主题

主题偏好设置会自动保存到本地存储，下次打开应用时会自动应用。

## 版本历史

- **v0.2.0** - 侧边栏重构、主题系统、托盘集成、全局快捷键、扫描逻辑重构
- **v0.1.0** - 初始版本

## License

MIT
