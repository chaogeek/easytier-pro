// 更新机制模块
// 负责两个独立的更新检查：
// 1. App 自更新 — 检查 GitHub Releases API (chaogeek/easytier-pro)
// 2. easytier-core 版本检查 — 检查 GitHub Releases API (EasyTier/EasyTier)
// 3. App 静默安装 — 下载 DMG → 挂载 → 替换 → 重启

use anyhow::{Context, Result};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use tauri::Emitter;
use tokio_stream::StreamExt;

/// GitHub Release API 响应结构（App 自更新和 core 更新通用）
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubRelease {
    tag_name: String,
    body: Option<String>,
    html_url: Option<String>,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

/// App 更新检查结果
#[derive(Debug, Clone, Serialize)]
pub struct AppUpdateInfo {
    /// 是否有新版本可用
    pub has_update: bool,
    /// 当前版本
    pub current_version: String,
    /// 最新版本号
    pub latest_version: String,
    /// 更新日志（Markdown）
    pub release_notes: String,
    /// 下载地址
    pub download_url: String,
}

/// easytier-core 更新检查结果
#[derive(Debug, Clone, Serialize)]
pub struct CoreUpdateInfo {
    /// 是否有新版本可用
    pub has_update: bool,
    /// 当前版本
    pub current_version: String,
    /// 最新版本号
    pub latest_version: String,
    /// 下载地址
    pub download_url: String,
}

/// App 自更新检查 URL
/// 由 Cloudflare Worker 代理 GitHub Releases API
/// 自定义域名: https://easytier-pro.782389.xyz
const APP_UPDATE_URL: &str = "https://easytier-pro.782389.xyz";

/// easytier-core GitHub Releases API URL
const CORE_UPDATE_URL: &str =
    "https://api.github.com/repos/EasyTier/EasyTier/releases/latest";

/// 请求超时（秒）
const REQUEST_TIMEOUT: u64 = 15;

/// 用户代理前缀
const USER_AGENT_PREFIX: &str = "EasyTierManager";

/// 去除版本号前导的 "v" 前缀
fn strip_v_prefix(version: &str) -> &str {
    version.strip_prefix('v').unwrap_or(version)
}

/// 比较两个版本号，如果 latest > current 返回 true
fn is_newer(current: &str, latest: &str) -> bool {
    let cur = Version::parse(strip_v_prefix(current)).ok();
    let lat = Version::parse(strip_v_prefix(latest)).ok();
    match (cur, lat) {
        (Some(c), Some(l)) => l > c,
        _ => false,
    }
}

/// 根据当前架构选择正确的 DMG 资产
/// Tauri 命名格式：easytier-pro_{version}_{arch}.dmg
fn select_dmg_asset(assets: &[GitHubAsset]) -> Option<&GitHubAsset> {
    let arch = std::env::consts::ARCH;
    let arch_keyword = match arch {
        "aarch64" => "aarch64",
        "arm" => "aarch64",
        _ => "x86_64",
    };

    // 优先匹配架构对应的 DMG（支持新旧两种命名格式）
    assets
        .iter()
        .find(|a| {
            a.name.ends_with(".dmg")
                && (a.name.contains(arch_keyword) || a.name.contains("arm64"))
        })
        .or_else(|| {
            // 兜底：取第一个 .dmg
            assets.iter().find(|a| a.name.ends_with(".dmg"))
        })
}

/// 通过架构关键词选择 easytier-core ZIP 资产
fn select_core_zip_asset(assets: &[GitHubAsset]) -> Option<&GitHubAsset> {
    let arch = std::env::consts::ARCH;
    let arch_keyword = match arch {
        "aarch64" => "aarch64",
        "arm" => "aarch64",
        _ => "x86_64",
    };

    assets
        .iter()
        .find(|a| {
            a.name.ends_with(".zip")
                && a.name.contains("macos")
                && a.name.contains(arch_keyword)
        })
}

/// 构造请求客户端
fn build_client(version: &str) -> Result<reqwest::blocking::Client> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        format!("{}/{}", USER_AGENT_PREFIX, version)
            .parse()
            .unwrap(),
    );

    reqwest::blocking::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT))
        .build()
        .context("创建 HTTP 客户端失败")
}

/// 检查 App 自更新
/// 通过 CF Worker 获取最新版本信息（避免 GitHub API 限流）
pub fn check_app_update(current_version: &str) -> Result<AppUpdateInfo> {
    let client = build_client(current_version)?;

    let release: GitHubRelease = client
        .get(APP_UPDATE_URL)
        .send()
        .context("获取 App 更新信息失败")?
        .json()
        .context("解析 App 更新信息失败")?;

    let latest_version = strip_v_prefix(&release.tag_name).to_string();
    let has_update = is_newer(current_version, &release.tag_name);

    let download_url = select_dmg_asset(&release.assets)
        .map(|a| a.browser_download_url.clone())
        .unwrap_or_default();

    Ok(AppUpdateInfo {
        has_update,
        current_version: current_version.to_string(),
        latest_version,
        release_notes: release.body.unwrap_or_default(),
        download_url,
    })
}

/// 检查 easytier-core 更新
/// 从 GitHub Releases API 获取最新版本
pub fn check_core_update(current_version: &str) -> Result<CoreUpdateInfo> {
    let client = build_client(current_version)?;

    let release: GitHubRelease = client
        .get(CORE_UPDATE_URL)
        .send()
        .context("获取 easytier-core 更新信息失败")?
        .json()
        .context("解析 easytier-core 更新信息失败")?;

    let latest_version = strip_v_prefix(&release.tag_name).to_string();
    let has_update = is_newer(current_version, &release.tag_name);

    let download_url = select_core_zip_asset(&release.assets)
        .map(|a| a.browser_download_url.clone())
        .unwrap_or_default();

    Ok(CoreUpdateInfo {
        has_update,
        current_version: current_version.to_string(),
        latest_version,
        download_url,
    })
}

/// 异步下载 DMG 到临时目录，通过 Tauri 事件推送实时进度
///
/// 事件名 "update-download-progress"，payload: { downloaded, total, percentage }
pub async fn download_dmg(
    download_url: &str,
    version: &str,
    app_handle: &tauri::AppHandle,
) -> Result<String> {
    let temp_dir =
        std::env::temp_dir().join(format!("easytier-pro-update-{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).context("创建临时目录失败")?;
    let dmg_path = temp_dir.join("update.dmg");

    // 构建异步 HTTP 客户端
    let client = reqwest::Client::builder()
        .user_agent(format!("{}/{}", USER_AGENT_PREFIX, version))
        .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT))
        .build()
        .context("创建 HTTP 客户端失败")?;

    let response = client
        .get(download_url)
        .send()
        .await
        .context("下载 DMG 失败")?;
    let total_size = response.content_length().unwrap_or(0);

    let mut downloaded: u64 = 0;
    let mut file = tokio::fs::File::create(&dmg_path)
        .await
        .context("创建临时文件失败")?;

    // 流式下载 + 推送进度事件
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("读取下载数据失败")?;
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk)
            .await
            .context("写入临时文件失败")?;
        downloaded += chunk.len() as u64;

        let pct = if total_size > 0 {
            (downloaded * 100 / total_size) as u8
        } else {
            0
        };
        let _ = app_handle.emit(
            "update-download-progress",
            serde_json::json!({
                "downloaded": downloaded,
                "total": total_size,
                "percentage": pct,
            }),
        );
    }

    // 确保 100% 事件
    let _ = app_handle.emit(
        "update-download-progress",
        serde_json::json!({
            "downloaded": downloaded,
            "total": total_size,
            "percentage": 100,
        }),
    );

    Ok(dmg_path.to_str().unwrap_or("").to_string())
}

/// 静默安装已下载的 DMG
///
/// 流程：
/// 1. 检查是否在 .app bundle 中
/// 2. 挂载 DMG → 找到 .app → 复制到 <当前>.new
/// 3. 写入孤儿 bash 替换脚本
/// 4. fire-and-forget 执行脚本 + exit(0)
pub fn install_dmg(dmg_path: &str) -> Result<()> {
    let dmg = std::path::PathBuf::from(dmg_path);
    if !dmg.exists() {
        anyhow::bail!("DMG 文件不存在: {}", dmg_path);
    }

    let current_exe = std::env::current_exe().context("无法获取当前可执行文件路径")?;
    let app_bundle = find_app_bundle(&current_exe);

    // Dev 模式：不在 .app bundle 中 → 用 Finder 打开 DMG 手动安装
    if app_bundle.is_none() {
        eprintln!("未在 .app bundle 中（dev 模式），打开 DMG 手动安装");
        Command::new("open")
            .arg(dmg_path)
            .spawn()
            .context("打开 DMG 失败")?;
        std::process::exit(0);
    }

    let app_bundle = app_bundle.unwrap();
    let app_bundle_new = app_bundle.with_extension("app.new");

    // 1. 挂载 DMG
    let mount_output = Command::new("hdiutil")
        .args([
            "attach", "-nobrowse", "-readonly", "-plist",
            dmg_path,
        ])
        .output()
        .context("挂载 DMG 失败")?;

    if !mount_output.status.success() {
        let stderr = String::from_utf8_lossy(&mount_output.stderr);
        anyhow::bail!("hdiutil attach 失败: {}", stderr);
    }

    let mount_point =
        parse_hdiutil_mount_point(&mount_output.stdout).context("解析 DMG 挂载点失败")?;

    // 2. 在挂载卷中找到 .app
    let new_app = find_app_in_dir(&mount_point).context("在 DMG 中未找到 .app")?;

    // 3. 复制 .app → <当前>.new
    let status = Command::new("cp")
        .args([
            "-R",
            new_app.to_str().unwrap_or(""),
            app_bundle_new.to_str().unwrap_or(""),
        ])
        .status()
        .context("复制新版本 .app 失败")?;
    if !status.success() {
        anyhow::bail!("cp -R 失败");
    }

    // 4. 孤儿替换脚本
    let fallback_tmp = std::path::PathBuf::from("/tmp");
    let temp_dir = dmg.parent().unwrap_or(&fallback_tmp);
    let script_path = temp_dir.join("install.sh");
    let script = format!(
        r#"#!/bin/bash
sleep 2
rm -rf "{}"
mv "{}" "{}"
rm -f "{}" 2>/dev/null
hdiutil detach "{}" -force 2>/dev/null
rm -rf "{}" 2>/dev/null
open "{}"
"#,
        app_bundle.display(),
        app_bundle_new.display(),
        app_bundle.display(),
        dmg_path,
        mount_point.display(),
        temp_dir.display(),
        app_bundle.display(),
    );

    std::fs::write(&script_path, script).context("写入安装脚本失败")?;
    Command::new("chmod")
        .args(["+x", script_path.to_str().unwrap_or("")])
        .status()
        .context("设置脚本权限失败")?;

    // 5. fire-and-forget 执行脚本
    Command::new("bash")
        .arg(script_path.to_str().unwrap_or(""))
        .spawn()
        .context("启动安装脚本失败")?;

    eprintln!("静默安装已启动，正在退出...");
    std::process::exit(0);
}

/// 从可执行文件路径向上查找 .app bundle
fn find_app_bundle(exe_path: &PathBuf) -> Option<PathBuf> {
    let mut path: PathBuf = exe_path.clone();
    while path.parent().is_some() {
        if path.extension().map_or(false, |ext| ext == "app") {
            return Some(path);
        }
        path = path.parent()?.to_path_buf();
    }
    None
}

/// 在目录中查找第一个 .app bundle
fn find_app_in_dir(dir: &PathBuf) -> Option<PathBuf> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "app") {
                return Some(path);
            }
        }
    }
    None
}

/// 解析 hdiutil attach -plist 输出中的挂载点
fn parse_hdiutil_mount_point(plist_output: &[u8]) -> Option<PathBuf> {
    // hdiutil 的 plist 输出中，mount-point 在 system-entities 数组的元素里
    // 简单方案：搜索 "mount-point" 字符串后的路径
    let text = String::from_utf8_lossy(plist_output);
    let marker = "<key>mount-point</key>";
    if let Some(pos) = text.find(marker) {
        let after = &text[pos + marker.len()..];
        // 下一个 <string> 标签内的内容就是挂载路径
        if let Some(start) = after.find("<string>") {
            let val_start = start + "<string>".len();
            if let Some(end) = after[val_start..].find("</string>") {
                let path_str = &after[val_start..val_start + end];
                return Some(PathBuf::from(path_str));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== strip_v_prefix 测试 ==========

    #[test]
    fn test_strip_v_prefix_with_v() {
        assert_eq!(strip_v_prefix("v1.2.3"), "1.2.3");
    }

    #[test]
    fn test_strip_v_prefix_without_v() {
        assert_eq!(strip_v_prefix("1.2.3"), "1.2.3");
    }

    #[test]
    fn test_strip_v_prefix_empty() {
        assert_eq!(strip_v_prefix(""), "");
    }

    // ========== is_newer 测试 ==========

    #[test]
    fn test_is_newer_latest_greater() {
        assert!(is_newer("0.0.1", "v0.0.2"));
        assert!(is_newer("1.0.0", "v2.0.0"));
        assert!(is_newer("1.0.0", "1.1.0"));
        assert!(is_newer("1.0.0", "1.0.1"));
    }

    #[test]
    fn test_is_newer_same_version() {
        assert!(!is_newer("0.0.1", "v0.0.1"));
        assert!(!is_newer("0.0.1", "0.0.1"));
        assert!(!is_newer("1.2.3", "v1.2.3"));
    }

    #[test]
    fn test_is_newer_current_greater() {
        assert!(!is_newer("0.0.2", "v0.0.1"));
        assert!(!is_newer("2.0.0", "v1.0.0"));
    }

    #[test]
    fn test_is_newer_both_no_v_prefix() {
        assert!(is_newer("0.0.1", "0.0.2"));
        assert!(!is_newer("0.0.2", "0.0.1"));
    }

    #[test]
    fn test_is_newer_invalid_version_returns_false() {
        // 无效版本号应安全返回 false（不 panic）
        assert!(!is_newer("invalid", "v1.0.0"));
        assert!(!is_newer("1.0.0", "invalid"));
        assert!(!is_newer("", "v1.0.0"));
    }

    // ========== select_dmg_asset 测试 ==========

    fn make_assets(names: &[&str]) -> Vec<GitHubAsset> {
        names
            .iter()
            .map(|name| GitHubAsset {
                name: name.to_string(),
                browser_download_url: format!("https://example.com/{}", name),
            })
            .collect()
    }

    #[test]
    fn test_select_dmg_prefers_arch() {
        let assets = make_assets(&[
            "easytier-pro_0.0.4_x86_64.dmg",
            "easytier-pro_0.0.4_aarch64.dmg",
        ]);

        let selected = select_dmg_asset(&assets).unwrap();
        if cfg!(target_arch = "aarch64") {
            assert!(selected.name.contains("aarch64"));
        } else {
            assert!(selected.name.contains("x86_64"));
        }
    }

    #[test]
    fn test_select_dmg_tauri_naming() {
        // 模拟 Tauri 实际构建产物命名
        let assets = make_assets(&[
            "easytier-pro_0.0.4_aarch64.dmg",
            "easytier-pro_0.0.4_amd64.deb",
            "easytier-pro_0.0.4_amd64.AppImage",
            "easytier-pro-0.0.4-1.x86_64.rpm",
        ]);

        let selected = select_dmg_asset(&assets).unwrap();
        // 应该选到 .dmg 而非 .deb/.AppImage
        assert!(selected.name.ends_with(".dmg"));
        if cfg!(target_arch = "aarch64") {
            assert!(selected.name.contains("aarch64"));
        }
    }

    #[test]
    fn test_select_dmg_old_naming_compat() {
        // 兼容旧命名 EasyTierManager-aarch64-v1.1.0.dmg
        let assets = make_assets(&[
            "EasyTierManager-arm64-v1.1.0.dmg",
            "EasyTierManager-x86_64-v1.1.0.dmg",
        ]);
        let selected = select_dmg_asset(&assets).unwrap();
        assert!(selected.name.ends_with(".dmg"));
    }

    #[test]
    fn test_select_dmg_fallback_to_first() {
        let assets = make_assets(&[
            "some-other-file.txt",
            "easytier-pro_0.0.4_aarch64.dmg",
        ]);

        let selected = select_dmg_asset(&assets).unwrap();
        assert!(selected.name.ends_with(".dmg"));
    }

    #[test]
    fn test_select_dmg_no_match() {
        let assets = make_assets(&["readme.md", "source.zip"]);
        assert!(select_dmg_asset(&assets).is_none());
    }

    // ========== select_core_zip_asset 测试 ==========

    #[test]
    fn test_select_core_zip_prefers_macos_arch() {
        let assets = make_assets(&[
            "easytier-windows-x86_64-v2.6.4.zip",
            "easytier-macos-aarch64-v2.6.4.zip",
            "easytier-macos-x86_64-v2.6.4.zip",
            "easytier-linux-x86_64-v2.6.4.zip",
        ]);

        let selected = select_core_zip_asset(&assets).unwrap();
        assert!(selected.name.contains("macos"));
        // 在当前架构机器上应选对应架构
        if cfg!(target_arch = "aarch64") {
            assert!(selected.name.contains("aarch64"));
        } else {
            assert!(selected.name.contains("x86_64"));
        }
    }

    #[test]
    fn test_select_core_zip_no_match() {
        let assets = make_assets(&["readme.md", "source.tar.gz"]);
        assert!(select_core_zip_asset(&assets).is_none());
    }

    // ========== 集成测试：完整更新检查流程 ==========

    #[test]
    fn test_update_info_structures() {
        // 测试结构体创建和序列化
        let app_info = AppUpdateInfo {
            has_update: true,
            current_version: "0.0.1".into(),
            latest_version: "0.0.2".into(),
            release_notes: "修复若干问题".into(),
            download_url: "https://example.com/app.dmg".into(),
        };

        let json = serde_json::to_string(&app_info).unwrap();
        assert!(json.contains("0.0.1"));
        assert!(json.contains("0.0.2"));
        assert!(json.contains("has_update"));

        let core_info = CoreUpdateInfo {
            has_update: false,
            current_version: "2.6.4".into(),
            latest_version: "2.6.4".into(),
            download_url: "".into(),
        };

        let json = serde_json::to_string(&core_info).unwrap();
        assert!(json.contains("2.6.4"));
        assert!(!json.contains("\"has_update\":true"));
    }
}
