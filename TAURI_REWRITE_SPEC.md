# EasyTierManager → Tauri 重构核心规范文档

> 本文档从当前 SwiftUI/AppKit 实现中提取所有关键架构决策、端点和行为细节，作为 Tauri 重构的技术参考。

---

## 目录

1. [macOS 提权助手](#1-macos-提权助手)
2. [更新机制](#2-更新机制)
3. [EasyTier 核心下载与管理](#3-easytier-核心下载与管理)
4. [应用生命周期管理](#4-应用生命周期管理)
5. [配置文件管理（TOML）](#5-配置文件管理toml)
6. [构建与打包](#6-构建与打包)
7. [窗口与 UI 规范](#7-窗口与-ui-规范)
8. [菜单栏与状态栏](#8-菜单栏与状态栏)
9. [CI/CD 发布流程](#9-cicd-发布流程)
10. [URL 端点汇总](#10-url-端点汇总)
11. [文件路径汇总](#11-文件路径汇总)

---

## 1. macOS 提权助手

### 1.1 架构概述

当前项目使用 **NSAppleScript + launchd** 方式提权，**未使用** Apple 标准的 `SMJobBless` API。原因：`SMJobBless` 需要付费 Apple Developer 证书签名，而项目使用 ad-hoc 签名（`CODE_SIGN_IDENTITY: "-"`）。

```
主应用（用户权限）
  │
  │ NSXPCConnection（Mach Service: "EasyTierHelper"）
  │
  ▼
EasyTierHelper（root 权限, launchd 守护进程）
  路径: /Library/PrivilegedHelperTools/EasyTierHelper
  │
  ├── startProcess(path, args) → PID    启动任意可执行文件
  ├── stopProcess(pid)         → Bool    发送 SIGTERM
  ├── forceStopProcess(pid)    → Bool    发送 SIGKILL
  ├── isProcessRunning(pid)    → Bool    检查进程存活
  ├── findProcess(pattern)     → [PID]   pgrep -af 搜索进程
  ├── stopAllProcesses()       → Bool    终止所有跟踪的进程
  ├── ping()                   → Bool    健康检查
  └── getVersion()             → String  获取 helper 版本
```

### 1.2 Helper 安装流程（NSAppleScript 方式）

这是 Tauri 重构时最重要的 macOS 特有逻辑。需要 Rust 侧通过 `tauri-plugin-shell` 或自定义 Tauri command 实现。

**安装步骤（等效 shell 脚本）：**

```bash
# 1. 创建目标目录
mkdir -p /Library/PrivilegedHelperTools

# 2. 从 app bundle 复制 helper 二进制
cp "App.app/Contents/Library/HelperTools/EasyTierHelper" \
   "/Library/PrivilegedHelperTools/EasyTierHelper"

# 3. 设置权限
chmod +x /Library/PrivilegedHelperTools/EasyTierHelper
chown root:wheel /Library/PrivilegedHelperTools/EasyTierHelper

# 4. 安装 launchd plist
cp "plist.xml" "/Library/LaunchDaemons/EasyTierHelper.plist"
chown root:wheel /Library/LaunchDaemons/EasyTierHelper.plist
chmod 644 /Library/LaunchDaemons/EasyTierHelper.plist

# 5. 加载守护进程
launchctl load /Library/LaunchDaemons/EasyTierHelper.plist
```

**提权方式：** 上述命令通过 `osascript -e 'do shell script "..." with administrator privileges'` 执行，macOS 会弹出标准管理员认证对话框。

**Tauri 实现建议：**
- 使用 `macos-privileges` crate 或执行 `osascript` 命令来提权
- 使用 `tauri-plugin-shell` 的 `open` 或 `execute` API 运行 shell 脚本
- 或将 helper 改为 Rust 编写的二进制（可作为 Tauri sidecar），使其更轻量

### 1.3 launchd Plist 配置

**安装到系统时的格式（`/Library/LaunchDaemons/EasyTierHelper.plist`）：**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>EasyTierHelper</string>
    <key>MachServices</key>
    <dict>
        <key>EasyTierHelper</key>
        <true/>
    </dict>
    <key>Program</key>
    <string>/Library/PrivilegedHelperTools/EasyTierHelper</string>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
```

关键点：
- `RunAtLoad: true` — 系统启动时自动运行
- `KeepAlive: true` — 崩溃后自动重启
- `MachServices` — 注册为 XPC Mach Service（如果 Tauri 不用 XPC 则可用其他 IPC 方式替代，如 Unix socket）

### 1.4 Helper 的代码签名防护

当前 Helper 在运行时动态获取自身 Team ID，并设置 XPC 连接要求：

```
identifier "com.easytier.manager" and certificate leaf[subject.OU] = "<TEAM_ID>"
```

这确保只有同团队签名的应用可以连接 Helper。**Tauri 重构时：** 如果改用其他 IPC（如 Unix socket + 文件权限控制），可用 socket 文件的 `root:wheel` 权限 + 路径白名单来替代代码签名校验。

### 1.5 关键路径常量

| 用途 | 路径 |
|------|------|
| Helper 安装位置 | `/Library/PrivilegedHelperTools/EasyTierHelper` |
| launchd plist 位置 | `/Library/LaunchDaemons/EasyTierHelper.plist` |
| App 内嵌 Helper 位置 | `Contents/Library/HelperTools/EasyTierHelper` |

---

## 2. 更新机制

### 2.1 架构概览

项目有 **两套独立的更新机制**：

| 更新目标 | 检查端点 | 实现位置 |
|----------|----------|----------|
| App 自身 | `https://easytier.782389.xyz` | `UpdateService.swift` |
| easytier-core/cli | `https://api.github.com/repos/EasyTier/EasyTier/releases/latest` | `SettingsView.swift` |

### 2.2 App 自更新流程

```
checkForUpdates()
  │
  ├── GET https://easytier.782389.xyz
  │   User-Agent: EasyTierManager/{version}
  │   timeout: 15s
  │
  ├── 响应格式（GitHub Release API 兼容）:
  │   {
  │     "tag_name": "v1.1.0",
  │     "body": "Release notes...",
  │     "html_url": "https://...",
  │     "assets": [
  │       {
  │         "name": "EasyTierManager-arm64-v1.1.0.dmg",
  │         "browser_download_url": "https://..."
  │       }
  │     ]
  │   }
  │
  ├── 版本比较: tag_name vs AppVersion.current (numeric compare, 去 v 前缀)
  │
  └── downloadAndInstall()
        │
        ├── 1. 根据架构选择 DMG 资产
        │     arm64 → 文件名含 "arm64" 或 "aarch64"
        │     x86_64 → 文件名含 "x86_64" 或 "x64" 或不含 arm 关键词
        │     兜底 → 任意 .dmg
        │
        ├── 2. URLSession 下载（带进度回调）
        │
        ├── 3. silentInstall():
        │     ├── hdiutil attach -nobrowse -readonly -plist {dmg}
        │     ├── 解析 plist 获取挂载点
        │     ├── 在挂载卷中找 .app bundle
        │     ├── cp -R {new.app} {currentPath}.new
        │     ├── 停止所有子进程 + 断开 helper
        │     ├── 写入 fire-and-forget bash 脚本:
        │     │   #!/bin/bash
        │     │   sleep 3
        │     │   rm -rf "{old.app}"
        │     │   mv "{old.app}.new" "{old.app}"
        │     │   rm -f "{dmg}" 2>/dev/null
        │     │   hdiutil detach "{mountPoint}" -force 2>/dev/null
        │     │   open "{old.app}"
        │     ├── Process.run(bash脚本) — fire & forget
        │     └── exit(0) — 立即退出当前进程
        │
        └── 错误兜底: NSWorkspace.shared.open(dmg) → Finder 打开 DMG 手动安装
```

### 2.3 静默安装的关键技术细节

1. **`exit(0)` vs `NSApp.terminate()`：** 使用 `exit(0)` 因为 `NSApp.terminate()` 在 bundle 被重命名后（macOS 14+）可能卡住。
2. **孤儿进程：** bash 替换脚本在父进程 `exit(0)` 后变为孤儿（reparent 到 launchd），继续运行完成替换。
3. **运行中替换 bundle：** macOS 允许移动/重命名正在运行的 `.app` 包，因为二进制文件保持其 fd 到可执行文件。
4. **DMG 挂载：** `hdiutil attach -nobrowse` 防止 Finder 弹窗，`-readonly` 防止意外修改，`-plist` 输出机器可解析。

### 2.4 easytier-core 更新流程（独立于 App 更新）

```
检查: GET https://api.github.com/repos/EasyTier/EasyTier/releases/latest
      User-Agent: EasyTierManager/{version}

版本比较: tag_name vs 当前 core --version 输出

下载: https://github.com/EasyTier/EasyTier/releases/download/{tag}/easytier-macos-{arch}-v{version}.zip

安装:
  1. 解压 ZIP 到临时目录
  2. cp easytier-core easytier-cli → App.app/Contents/Helpers/
  3. chmod 755 两个文件
  4. 清除版本缓存
```

---

## 3. EasyTier 核心下载与管理

### 3.1 下载源

| 文件 | 下载地址 | 硬编码版本 |
|------|----------|-----------|
| easytier-core | `https://github.com/EasyTier/Easytier/releases/download/v{version}/easytier-macos-{arch}-v{version}.zip` | `2.6.4` |
| easytier-cli | （同上 ZIP） | `2.6.4` |

**架构映射：**
- `uname -m` = `arm64` → URL 中 arch 为 `aarch64`
- `uname -m` = `x86_64` → URL 中 arch 为 `x86_64`

**缓存策略：** 下载的 ZIP 缓存到 `~/Library/Caches/com.easytier.manager/easytier-binaries/`，后续构建复用。

### 3.2 构建时嵌入

```
App.app/Contents/
├── Helpers/
│   ├── easytier-core    (从 GitHub 下载, ~21MB)
│   └── easytier-cli     (从 GitHub 下载, ~9MB)
├── Library/
│   ├── HelperTools/
│   │   └── EasyTierHelper    (编译产物, ~119KB)
│   └── LaunchDaemons/
│       └── EasyTierHelper.plist
├── MacOS/
│   └── EasyTierManager
└── Resources/
    ├── AppIcon.icns
    └── Assets.car
```

### 3.3 运行时路径解析

```swift
// 当前 Swift 实现 → Tauri 等效
let helpersPath = Bundle.main.bundlePath + "/Contents/Helpers"
let corePath    = helpersPath + "/easytier-core"
let cliPath     = helpersPath + "/easytier-cli"

// Tauri 等效（使用 tauri::api::path 或 process::current_dir）
// 通过 app.resource_dir() 或 env!("CARGO_MANIFEST_DIR") 定位
```

### 3.4 进程管理关键行为

| 操作 | 方法 | 细节 |
|------|------|------|
| 启动 core | `easytier-core -c <config1.toml> -c <config2.toml>` | 支持多配置文件，通过 Helper 以 root 权限启动 |
| 停止 core | SIGTERM → 等 5s → SIGKILL | 优雅关闭优先 |
| 健康检查 | 每 30s 检查 PID 存活 | 连续 5 次失败 → 标记 error |
| 重启策略 | 失败后自动重启 | 最多 5 次尝试 |
| 睡眠处理 | 睡眠前停止 core + 断开 helper | 唤醒后重连 + 重启（最多 5 次重试，递增延迟） |
| CLI 查询 | `easytier-cli -o json peer list` | 直接执行（不通过 Helper），15s 超时 |
| CLI 查询 | `easytier-cli -o json node info` | 同上 |

### 3.5 版本获取

```bash
easytier-core --version
# → 输出第一行为版本号（用于显示和更新比较）
```

---

## 4. 应用生命周期管理

### 4.1 启动流程

```
App 启动
  ├── 初始化 NavigationVM, NetworkStore, EasyTierService
  ├── 创建 MainWindowController（860×660 窗口）
  ├── 创建 StatusBarController（菜单栏图标）
  ├── 根据设置决定 Dock 图标 vs 仅菜单栏模式
  ├── 注册 SIGTERM/SIGINT 信号处理器（优雅退出）
  ├── 注册 NSWorkspace 睡眠/唤醒通知
  └── 异步:
        ├── await EasyTierHelperManager.installAndConnect()
        │     ├── 尝试连接已有 Helper
        │     ├── 失败 → 自动安装 Helper（弹出管理员认证）
        │     └── 连接成功 → isHelperConnected = true
        └── await performAutoConnect()（自动连接标记的网络）
```

### 4.2 退出流程（SIGTERM/SIGINT）

```swift
signal(SIGTERM) { _ in
    // 1. 停止所有 easytier-core 进程
    // 2. 停止健康检查
    // 3. 断开 Helper 连接
    // 4. exit(0)
}
```

### 4.3 睡眠/唤醒处理

| 事件 | 操作 |
|------|------|
| `NSWorkspace.willSleepNotification` | 停止 core → 断开 helper → 保存活跃配置列表 |
| `NSWorkspace.didWakeNotification` | 等待 2s → 重连 helper → 重启 core（最多 5 次，递增延迟） |

**Tauri 实现建议：** 使用 `tauri-plugin-shell` 配合 Rust 侧的 `tauri::RunEvent` 或监听系统事件来实现睡眠/唤醒处理。Rust 侧可用 `objc` crate 桥接 `NSWorkspace` 通知。

### 4.4 图标模式切换

支持两种运行模式（`UserDefaults` 持久化）：

| 模式 | `NSApp.activationPolicy` | 表现 |
|------|--------------------------|------|
| 仅菜单栏 | `.accessory` | 无 Dock 图标，仅菜单栏图标 |
| 菜单栏 + Dock | `.regular` | 同时显示 Dock 图标和菜单栏图标 |

**约束：** 至少一个入口可见（不能同时隐藏 Dock 和菜单栏图标）。

---

## 5. 配置文件管理（TOML）

### 5.1 存储路径

```
~/Library/Application Support/EasyTierManager/
├── networks.json                          # 网络列表
└── configs/
    └── config_{uuid}.toml                 # 每个网络的 TOML 配置
```

### 5.2 TOML 格式

```toml
hostname = "my-host"
instance_name = "my-network"
ipv4 = "10.144.0.1"
dhcp = true

listeners = [
    "tcp://0.0.0.0:11010",
    "udp://0.0.0.0:11010"
]

[network_identity]
network_name = "my-network"
network_secret = "my-secret"

[[peer]]
uri = "tcp://peer.example.com:11010"

[flags]
accept_dns = true
latency_first = true
private_mode = false
```

### 5.3 导入/导出

| 功能 | 实现 |
|------|------|
| 导入 | NSOpenPanel → `.toml`/`.yaml`/`.plainText` → parse → 填充编辑表单 |
| 单网络导出 | NSSavePanel → 复制 TOML 文件到选定位置 |
| 全部导出 | NSOpenPanel（目录模式）→ 复制所有 TOML 文件 |

**Tauri 实现：** 使用 `tauri-plugin-dialog` 做文件选择，Rust 侧用 `toml` crate 做解析。

### 5.4 网络列表持久化（JSON）

```json
[
  {
    "id": "uuid",
    "name": "我的网络",
    "configPath": "/path/to/config.toml",
    "isAutoConnect": true,
    "status": "disconnected"
  }
]
```

状态枚举：`disconnected` | `connecting` | `connected` | `error`

---

## 6. 构建与打包

### 6.1 当前构建流程

```
build-release.sh
  │
  ├── 1. 读取 VERSION 文件 → APP_VERSION
  │
  ├── 2. download_easytier()
  │     从 GitHub 下载 easytier-core v2.6.4 + easytier-cli
  │     缓存到 ~/Library/Caches/com.easytier.manager/easytier-binaries/
  │     解压到 Helpers/
  │
  ├── 3. scripts/generate-version.sh
  │     读取 VERSION → 写入 Sources/...Version.generated.swift
  │
  ├── 4. xcodegen generate (MARKETING_VERSION=$APP_VERSION)
  │
  ├── 5. xcodebuild (Release)
  │     ├── EasyTierManager scheme → App.app
  │     └── EasyTierHelper scheme → Helper 二进制
  │
  ├── 6. 嵌入 Helpers:
  │     cp easytier-core/cli → App.app/Contents/Helpers/
  │     cp EasyTierHelper → App.app/Contents/Library/HelperTools/
  │
  └── 7. hdiutil create → EasyTierManager-{arch}-v{version}.dmg
```

### 6.2 Tauri 等效实现建议

```
构建脚本:
  ├── 1. 读取 VERSION
  ├── 2. 下载 easytier-core/cli 到指定目录
  ├── 3. 设置 tauri.conf.json 的 version 字段
  ├── 4. cargo tauri build → .dmg
  └── 5. 在 tauri.conf.json 的 bundle.resources 中包含:
        Helpers/easytier-core
        Helpers/easytier-cli
        （Helper 改为 Rust sidecar 或独立二进制）
```

### 6.3 DMG 命名规范

```
EasyTierManager-{arch}-v{version}.dmg

示例:
  EasyTierManager-aarch64-v1.1.0.dmg
  EasyTierManager-x86_64-v1.1.0.dmg
```

### 6.4 部署目标

- macOS 最低版本：**13.0 (Ventura)**
- 架构：Universal 或 分别构建 arm64 / x86_64

---

## 7. 窗口与 UI 规范

### 7.1 窗口

| 属性 | 值 |
|------|-----|
| 尺寸 | **860 × 660** |
| 缩放 | **禁用** |
| 最小化按钮 | **隐藏** |
| 标题栏 | **透明**（`.fullSizeContentView`） |
| 背景色 | Asset Catalog: `Background` / `Background1` / `Background2` |

### 7.2 导航结构

```
左侧栏（200pt）
├── 网络
│   ├── 连接 (ConnectionsView) — Cmd+1
│   └── 节点 (NodesView)       — Cmd+2
└── 设置
    └── 设置 (SettingsView)    — Cmd+3
```

### 7.3 UI 语言

**全部中文：**

| 英文概念 | 中文显示 |
|----------|----------|
| Network | 网络 |
| Connections | 连接 |
| Nodes | 节点 |
| Settings | 设置 |
| Disconnected | 未连接 |
| Connected | 已连接 |
| Error | 错误 |
| Peer | 节点/对等点 |

### 7.4 组件约定

- **边框：** 使用自定义 `border(width:edges:color:)` 修饰符（非系统 Divider）
- **卡片：** 使用 `SettingsSection` 组件做分组卡片布局
- **代码风格：** 不加注释，保持简洁（`AGENTS.md` 中要求）

---

## 8. 菜单栏与状态栏

### 8.1 菜单栏应用菜单

```
EasyTierManager (App 菜单)
├── 关于 EasyTierManager
├── ──────────────
├── 隐藏
├── 隐藏其他
├── 显示全部
├── ──────────────
└── 退出
```

### 8.2 状态栏（NSStatusItem）

菜单栏图标显示：
- 网络状态概览
- 每个网络的独立切换子菜单
- 节点信息（IP/延迟/类型）
- 全部连接 / 全部断开
- 显示窗口
- 退出

**Tauri 实现：** 使用 `tauri-plugin-tray`（或 `tray-icon` crate）配合 `tauri::menu` 构建系统托盘，功能上需保持等效。

---

## 9. CI/CD 发布流程

### 9.1 GitHub Actions (`release.yml`)

```
触发: git tag v*

流程:
  1. checkout 代码
  2. 验证 VERSION 文件 == tag 版本（不匹配则报错中断）
  3. 安装 xcodegen
  4. 运行 build-release.sh
  5. 上传 DMG 为 artifact
  6. 创建 GitHub Release（softprops/action-gh-release@v2）
     - 自动生成 release notes
     - 附加 DMG 文件
  7. 更新 Homebrew Tap:
     - 计算 DMG SHA256
     - 运行 scripts/update-homebrew-tap.sh
     - 更新 begitcn/homebrew-tap 中的 easytiermanager.rb
```

### 9.2 Homebrew Cask 部署

```
仓库: github.com/begitcn/homebrew-tap
文件: Casks/easytiermanager.rb

cask 内容:
  - name: "EasyTierManager"
  - url: GitHub Release DMG 地址（分 arm64 / x86_64）
  - sha256: 分别提供 arm64 和 x86_64 的校验值
  - app target: "EasyTierManager.app"
```

### 9.3 Tauri 适配

GitHub Actions 改为使用 `tauri-action`：
```yaml
- uses: tauri-apps/tauri-action@v0
  with:
    tagName: v__VERSION__
    releaseName: 'EasyTierManager v__VERSION__'
    releaseBody: 'See CHANGELOG'
    releaseDraft: false
```

---

## 10. URL 端点汇总

| 用途 | URL | 方法 | 格式 |
|------|-----|------|------|
| App 自更新检查 | `https://easytier.782389.xyz` | GET | GitHub Release JSON |
| easycore 版本检查 | `https://api.github.com/repos/EasyTier/EasyTier/releases/latest` | GET | GitHub API JSON |
| easycore 下载 | `https://github.com/EasyTier/EasyTier/releases/download/v{version}/easytier-macos-{arch}-v{version}.zip` | GET | ZIP |
| App DMG 下载（Homebrew） | `https://github.com/begitcn/EasyTierManager/releases/download/v{version}/EasyTierManager-{arch}-v{version}.dmg` | GET | DMG |
| Homebrew Tap | `https://github.com/begitcn/homebrew-tap.git` | git clone | Git repo |
| GitHub Issues | `https://github.com/begitcn/EasyTierManager/issues` | - | 网页 |

### App 自更新 API 响应格式

```json
{
  "tag_name": "v1.1.0",
  "body": "Release notes markdown...",
  "html_url": "https://github.com/begitcn/EasyTierManager/releases/tag/v1.1.0",
  "assets": [
    {
      "name": "EasyTierManager-aarch64-v1.1.0.dmg",
      "browser_download_url": "https://github.com/begitcn/EasyTierManager/releases/download/v1.1.0/EasyTierManager-aarch64-v1.1.0.dmg"
    },
    {
      "name": "EasyTierManager-x86_64-v1.1.0.dmg",
      "browser_download_url": "https://github.com/begitcn/EasyTierManager/releases/download/v1.1.0/EasyTierManager-x86_64-v1.1.0.dmg"
    }
  ]
}
```

---

## 11. 文件路径汇总

### 11.1 系统路径（硬编码）

| 路径 | 用途 |
|------|------|
| `/Library/PrivilegedHelperTools/EasyTierHelper` | Helper 二进制安装位置 |
| `/Library/LaunchDaemons/EasyTierHelper.plist` | Helper launchd plist 位置 |

### 11.2 用户数据路径（相对于 `~`）

| 路径 | 用途 |
|------|------|
| `~/Library/Application Support/EasyTierManager/` | 应用数据根目录 |
| `~/Library/Application Support/EasyTierManager/networks.json` | 网络列表持久化 |
| `~/Library/Application Support/EasyTierManager/configs/config_{uuid}.toml` | 各网络 TOML 配置 |
| `~/Library/Caches/com.easytier.manager/easytier-binaries/` | easycore 二进制缓存 |
| `~/Library/Preferences/com.easytier.manager.plist` | UserDefaults 设置 |

### 11.3 App Bundle 内部路径

| 路径 | 内容 |
|------|------|
| `Contents/Helpers/easytier-core` | EasyTier 守护进程 |
| `Contents/Helpers/easytier-cli` | EasyTier CLI 工具 |
| `Contents/Library/HelperTools/EasyTierHelper` | Privileged Helper |
| `Contents/Library/LaunchDaemons/EasyTierHelper.plist` | Helper launchd plist |
| `Contents/MacOS/EasyTierManager` | 主程序二进制 |
| `Contents/Resources/AppIcon.icns` | 应用图标 |
| `Contents/Resources/Assets.car` | 编译后的颜色/图片资源 |

### 11.4 项目源文件结构

```
/
├── VERSION                    # 单一版本号源（纯文本: "1.1.0"）
├── build-release.sh           # 完整发布构建脚本
├── generate-xcodeproj.sh      # Xcode 项目生成（Tauri 不需要）
├── project.yml                # XcodeGen 配置（Tauri 不需要）
├── Package.swift              # SPM 定义（Tauri 不需要）
├── tauri.conf.json            # ← Tauri 项目新增，替代上面三个
│
├── Helpers/                   # 运行时二进制（gitignore）
│   ├── easytier-core
│   └── easytier-cli
│
├── src-tauri/                 # ← Tauri Rust 后端
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── src/
│   │   ├── main.rs            # 入口
│   │   ├── helper.rs          # 提权助手（新实现）
│   │   ├── update.rs          # 更新服务（新实现）
│   │   └── easycore.rs        # EasyTier 管理（新实现）
│   └── icons/
│
├── src/                       # ← Tauri 前端
│   ├── App.svelte             # 或 React/Vue 等
│   ├── lib/
│   │   ├── components/
│   │   ├── stores/
│   │   └── views/
│   └── assets/
│
├── scripts/
│   ├── generate-version.sh
│   ├── update-homebrew-tap.sh
│   └── download-easycore.sh   # ← 新建：专门下载 easycore 的脚本
│
├── .github/workflows/
│   └── release.yml
│
└── README.md
```

---

## 附录 A: Tauri 重构关键决策点

1. **提权助手：** 考虑将 Swift XPC Helper 改为 Rust sidecar 二进制，通过 Unix socket 或 stdin/stdout 与主进程通信。安装方式保持 NSAppleScript + launchd（兼容无开发者证书场景）。

2. **进程管理：** Rust 侧使用 `std::process::Command` 管理 easycore 生命周期，替代 NSXPC `Process` 调用。

3. **自更新：** 保持相同的更新服务器端点和静默安装流程。Tauri 的 `tauri-plugin-updater` 可作为替代方案，但需要调整服务器响应格式。

4. **配置文件：** Rust 的 `toml` + `serde_json` crate 替代 Swift 的手写 TOML 解析器。

5. **系统睡眠/唤醒：** Rust 侧通过 `objc` crate 或 `tauri::RunEvent` 监听，可以更简洁地处理。

6. **菜单栏图标：** 使用 `tauri-plugin-tray` 替代 NSStatusItem。

---

## 附录 B: 版本号管理约定

- **单一源：** `VERSION` 文件为唯一的版本号来源
- **格式：** 纯语义版本号（如 `1.1.0`），无 `v` 前缀
- **CI 校验：** git tag（如 `v1.1.0`）必须与 `VERSION` 文件内容一致
- **自动生成：** 构建时自动生成版本常量代码文件

---

> **文档版本：** v1.0  
> **基于代码：** EasyTierManager v1.1.0 (commit: 6d72be3)  
> **最后更新：** 2026-07-06
