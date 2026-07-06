# easytier-pro — 智能体指令

## 项目概述

EasyTierManager 的 Tauri v2 重构版本，一个用于管理 [EasyTier](https://github.com/EasyTier/EasyTier) 组网 VPN 节点的 macOS 桌面应用。原 SwiftUI/AppKit 应用正在移植为 **Rust（Tauri 后端）+ React 19 + TypeScript（Vite 前端）**。

**实现任何功能前，务必先阅读 `TAURI_REWRITE_SPEC.md`。** 该文档记录了原 Swift 应用的每一个架构决策、端点、文件路径和行为细节。

## 技术栈

| 层 | 技术 |
|-------|-----------|
| 桌面框架 | Tauri v2 |
| 后端 | Rust（2021 版） |
| 前端 | React 19 + TypeScript 5.8 |
| UI 组件库 | **Material UI (MUI)** — 所有按钮、卡片、输入框、弹窗等均基于 MUI |
| 路由 | **react-router** — 用于左侧菜单栏页面切换 |
| 数据请求 | **SWR** — 用于前端高效、定时轮询后端数据（延迟、流量统计等） |
| 打包工具 | Vite 7 |
| 包管理器 | pnpm |
| 目标系统 | macOS ≥ 13.0（主力），Windows/Linux 后续支持 |

## 构建与运行命令

```bash
pnpm install                 # 安装前端依赖
pnpm dev                     # Vite 开发服务器（端口 1420）
pnpm build                   # tsc + vite build（仅前端）
pnpm tauri dev               # 完整 Tauri 开发模式（Rust + 前端）
pnpm tauri build             # 生产构建
```

## 项目结构

```
src/                          # React 前端
  App.tsx                     # 根组件（当前为模板占位代码）
  main.tsx                    # React 入口
  App.css                     # 全局样式

src-tauri/                    # Tauri Rust 后端
  src/
    lib.rs                    # Tauri 命令与应用初始化
    main.rs                   # 二进制入口
  Cargo.toml                  # Rust 依赖
  tauri.conf.json             # Tauri 配置
  capabilities/default.json   # 权限模型（Tauri v2）
  icons/                      # 应用图标

TAURI_REWRITE_SPEC.md         # 核心参考文档（731 行）
```

## 关键架构决策（来自重构规范）

1. **macOS 提权机制** — 使用 `osascript` + `launchd`（非 SMJobBless）安装特权助手。特权助手以 root 权限运行，负责 easytier-core 的进程管理。详见规范 §1。

2. **双更新系统** — 应用自更新（来自 `easytier.782389.xyz`）和 easytier-core 更新（来自 GitHub Releases）。两者需独立实现。详见规范 §2。

3. **TOML 配置管理** — 每个网络对应一个 `config_{uuid}.toml` 文件，存放在 `~/Library/Application Support/EasyTierManager/configs/`。网络列表以 JSON 格式保存在 `networks.json`。详见规范 §5。

4. **进程生命周期** — easytier-core 通过特权助手以 `-c config1.toml -c config2.toml` 参数启动。先发 SIGTERM，5 秒后发 SIGKILL。每 30 秒健康检查。需处理系统睡眠/唤醒。详见规范 §3-4。

5. **窗口规范** — 860×660，不可调整大小，透明标题栏。左侧边栏（200pt）含三个视图：连接、节点、设置。详见规范 §7。

6. **界面为中文** — 所有标签、菜单、状态文本一律中文。详见规范 §7.3。

## 编码约定

- **前端**：TypeScript 严格模式，不允许未使用的局部变量/参数。模块解析使用 bundler 模式。
- **UI 组件**：统一使用 MUI（Material UI）组件，不手写原生 HTML 按钮/输入框/卡片/弹窗等。遵循 MUI 的 `sx` prop 或 `styled()` 方式做样式定制。
- **路由**：使用 react-router 实现左侧菜单栏三个页面的切换（连接/节点/设置），保持 URL 与当前视图同步。
- **数据请求**：使用 SWR 做数据获取和定时轮询。对需要实时刷新的数据（如节点延迟、流量统计、连接状态）用 SWR 的 `refreshInterval` 自动轮询；对一次性操作（如启动/停止 core）用 Tauri `invoke` 直接调用。
- **后端**：使用 `serde` 做序列化，`toml` crate 做配置解析。IPC 优先使用 `tauri::command`。
- **注释用中文** — 所有代码注释、文档使用中文。
- **硬编码版本**：easytier-core 默认下载版本为 v2.6.4。
- **应用标识**：`com.chaogeek.easytier-pro`（原应用为 `com.easytier.manager`）。

## 后续需要添加的 Rust Crate

后端开发推进过程中，预计会用到：
- `toml` — TOML 配置解析
- `serde` / `serde_json` — 序列化（已添加）
- `reqwest` 或 `ureq` — 用于更新检查的 HTTP 客户端
- `tokio` — 异步运行时（Tauri 已内置）
- `objc` / objc2 — macOS 系统事件桥接（睡眠/唤醒）

## 注意事项 / 踩坑点

- **macOS bundle 路径**：easytier-core/cli 放在 `Contents/Helpers/`，不是 Tauri 默认的资源路径。可能需要自定义 `tauri.conf.json` 的 `resources` 配置。
- **静默更新**：原应用使用 `exit(0)` + 孤儿 bash 脚本进行自替换。Tauri 的 updater 插件未必支持此模式，可能需要自定义实现。
- **Vite 开发端口固定**为 1420（strictPort: true）。Tauri 开发模式依赖此端口。
- **尚未初始化 git 仓库** — 这是一个全新项目，准备好后初始化。
