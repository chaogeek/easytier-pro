<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" width="128" alt="easytier-pro" />
</p>

<h1 align="center">easytier-pro</h1>

<p align="center">
  <strong>EasyTier 网络管家</strong> — 组网 VPN 桌面管理工具
</p>

<p align="center">
  <a href="https://github.com/chaogeek/easytier-pro/releases"><img src="https://img.shields.io/github/v/release/chaogeek/easytier-pro?color=blue" alt="Release"></a>
  <a href="https://github.com/chaogeek/easytier-pro/actions"><img src="https://img.shields.io/github/actions/workflow/status/chaogeek/easytier-pro/release.yml?branch=main" alt="CI"></a>
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey" alt="Platform">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License">
</p>

---

## 简介

**easytier-pro** 是 [EasyTier](https://github.com/EasyTier/EasyTier) 的桌面管理客户端，提供图形化界面来管理组网节点、监控连接状态和流量数据。

EasyTier 是一个去中心化的点对点 VPN 组网工具，支持 UDP/HOLE PUNCHING/WIRE 等多种协议，可用于异地组网、远程办公、游戏联机等场景。easytier-pro 让你无需记忆命令行参数，通过直观的界面即可：

- 创建和管理多个网络
- 一键启动/停止 EasyTier 核心服务
- 实时查看节点延迟和流量统计
- 自动检测和安装核心更新

## 功能

- 🔌 **网络管理** — 创建、编辑、删除多个 EasyTier 网络配置
- 📡 **节点监控** — 实时查看所有节点状态、延迟和流量
- 🔄 **自动更新** — App 和 easytier-core 独立更新检测 + 进度条下载
- 🎛️ **设置管理** — 图形化配置 TOML 参数，无需手写配置文件
- 🌐 **多平台** — macOS / Windows / Linux 全平台支持
- 🇨🇳 **中文界面** — 全中文 UI，清晰直观

## 安装

### Homebrew（macOS 推荐）

```bash
brew install chaogeek/tap/easytier-pro
```

### 手动下载

从 [Releases](https://github.com/chaogeek/easytier-pro/releases) 页面下载对应平台的安装包：

| 平台 | 格式 |
|------|------|
| macOS (Apple Silicon) | `.dmg` |
| Windows | `.msi` |
| Linux | `.deb` |

### 从源码构建

**前置依赖：**

- [Rust](https://www.rust-lang.org/) ≥ 1.80
- [Node.js](https://nodejs.org/) ≥ 22
- [pnpm](https://pnpm.io/) ≥ 10
- macOS: Xcode Command Line Tools
- Linux: `libwebkit2gtk-4.1-dev libgtk-3-dev` 等

```bash
# 克隆仓库
git clone https://github.com/chaogeek/easytier-pro.git
cd easytier-pro

# 安装依赖
pnpm install

# 开发模式运行
pnpm tauri dev

# 生产构建
pnpm tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`。

## 使用指南

### 首次启动

1. 启动 easytier-pro
2. 进入「连接」页面，点击「新建网络」
3. 填写网络名称和基本参数
4. 点击「启动」运行 EasyTier 核心服务
5. 切换到「节点」页面查看连接状态

### 更新检测

1. 进入「设置」页面
2. 点击「检查更新」检测新版本
3. 如有新版本，点击「立即更新」下载
4. 下载完成后点击「立即重启」安装更新

## 技术栈

| 层 | 技术 |
|---|---|
| 桌面框架 | [Tauri v2](https://v2.tauri.app/) |
| 后端 | Rust 2021 |
| 前端 | React 19 + TypeScript 5.8 |
| UI 组件库 | [Material UI (MUI)](https://mui.com/) |
| 路由 | react-router v7 |
| 数据请求 | [SWR](https://swr.vercel.app/) |
| 打包工具 | Vite 7 |
| 包管理器 | pnpm |

## 项目结构

```
├── src/                        # React 前端
│   ├── App.tsx                 # 根组件 & 路由
│   ├── main.tsx                # 入口
│   ├── components/             # 公共组件
│   │   └── AppShell.tsx        # 应用外壳（侧边栏 + 内容区）
│   └── pages/                  # 页面
│       ├── Connections.tsx     # 连接管理
│       ├── Nodes.tsx           # 节点监控
│       └── Settings.tsx        # 设置 & 更新
├── src-tauri/                  # Tauri Rust 后端
│   ├── src/
│   │   ├── lib.rs              # Tauri 命令注册
│   │   ├── main.rs             # 二进制入口
│   │   ├── update.rs           # 更新检查 & 静默安装
│   │   └── version.rs          # 版本号常量
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/           # 权限配置
│   └── icons/                  # 应用图标
├── .github/workflows/          # CI/CD
│   └── release.yml             # 多平台构建 & Release
├── TAURI_REWRITE_SPEC.md       # 重构规范文档
└── VERSION                     # 当前版本号
```

## 开发

```bash
pnpm install        # 安装前端依赖
pnpm dev            # 仅启动 Vite 前端（端口 1420）
pnpm tauri dev      # 完整 Tauri 开发模式（前端 + Rust 后端）
pnpm tauri build    # 生产构建 & 打包
```

> **注意**：开发模式下更新安装功能受限（无 `.app` bundle），会自动打开 DMG 供手动安装。

## CI/CD

推送 `v*` 格式的 tag 自动触发多平台构建和发布：

```bash
# 1. 更新版本号
echo "0.0.3" > VERSION
# 同时更新: Cargo.toml, tauri.conf.json, package.json

# 2. 提交并打 tag
git add -A && git commit -m "v0.0.3"
git tag -a v0.0.3 -m "v0.0.3"
git push origin main && git push origin v0.0.3
```

CI 流水线：
1. **版本校验** — 检查 tag 与 VERSION 文件一致
2. **四平台构建** — macOS ARM64 / Windows / Linux
3. **创建 Release** — 上传安装包到 GitHub Releases
4. **Homebrew Tap** — 自动更新 Cask 文件

## 许可证

[MIT](LICENSE)

## 致谢

- [EasyTier](https://github.com/EasyTier/EasyTier) — 核心组网引擎
- [Tauri](https://tauri.app/) — 轻量级桌面应用框架
