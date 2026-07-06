// 更新机制模块
// 负责两个独立的更新检查：
// 1. App 自更新 — 检查 easytier.782389.xyz
// 2. easytier-core 版本检查 — 检查 GitHub Releases API

use anyhow::{Context, Result};
use semver::Version;
use serde::{Deserialize, Serialize};

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
const APP_UPDATE_URL: &str = "https://easytier.782389.xyz";

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
fn select_dmg_asset(assets: &[GitHubAsset]) -> Option<&GitHubAsset> {
    let arch = std::env::consts::ARCH;
    let arch_keyword = match arch {
        "aarch64" => "aarch64",
        "arm" => "aarch64",
        _ => "x86_64",
    };

    // 优先匹配架构对应的 DMG
    assets
        .iter()
        .find(|a| a.name.ends_with(".dmg") && a.name.contains(arch_keyword))
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
/// 从 easytier.782389.xyz 获取最新版本信息
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
