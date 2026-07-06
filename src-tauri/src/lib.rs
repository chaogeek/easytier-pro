mod update;
mod version;

use tauri::State;
use std::sync::Mutex;

/// 应用状态
struct AppState {
    /// easytier-core 当前版本号
    core_version: Mutex<String>,
    /// 已下载的更新 DMG 路径（None = 未下载）
    update_dmg_path: Mutex<Option<String>>,
}

/// 问候测试命令（开发阶段用）
#[tauri::command]
fn greet(name: &str) -> String {
    format!("你好，{}！来自 Rust 的问候！", name)
}

/// 检查 App 自身更新
#[tauri::command]
fn check_app_update(_state: State<AppState>) -> Result<update::AppUpdateInfo, String> {
    let version = version::APP_VERSION;
    update::check_app_update(version).map_err(|e| format!("检查更新失败：{}", e))
}

/// 检查 easytier-core 更新
#[tauri::command]
fn check_core_update(state: State<AppState>) -> Result<update::CoreUpdateInfo, String> {
    let core_ver = state.core_version.lock().unwrap().clone();
    let ver = if core_ver.is_empty() {
        "2.6.4"
    } else {
        &core_ver
    };
    update::check_core_update(ver).map_err(|e| format!("检查 core 更新失败：{}", e))
}

/// 获取应用版本号
#[tauri::command]
fn get_app_version() -> String {
    version::APP_VERSION.to_string()
}

/// 获取 easytier-core 当前版本号
#[tauri::command]
fn get_core_version(state: State<AppState>) -> String {
    let core_ver = state.core_version.lock().unwrap().clone();
    if core_ver.is_empty() {
        "2.6.4".to_string()
    } else {
        core_ver
    }
}

/// 下载 App 更新 DMG（带进度事件，异步流式下载）
///
/// 通过 Tauri 事件 "update-download-progress" 推送实时进度：
/// { downloaded: u64, total: u64, percentage: u8 }
#[tauri::command]
async fn download_app_update(
    download_url: String,
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let ver = version::APP_VERSION;
    let dmg_path = update::download_dmg(&download_url, ver, &app_handle)
        .await
        .map_err(|e| format!("下载更新失败：{}", e))?;

    *state.update_dmg_path.lock().unwrap() = Some(dmg_path);
    Ok(())
}

/// 静默安装已下载的更新
///
/// 挂载 DMG → 复制 .app → 孤儿脚本替换 → 退出重启
/// 成功后不会返回（process::exit）
#[tauri::command]
fn install_app_update(state: State<AppState>) -> Result<(), String> {
    let dmg_path = state
        .update_dmg_path
        .lock()
        .unwrap()
        .clone()
        .ok_or("没有已下载的更新包".to_string())?;

    update::install_dmg(&dmg_path).map_err(|e| format!("安装更新失败：{}", e))
    // 成功后不会执行到这里（process::exit 已退出）
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            core_version: Mutex::new(String::new()),
            update_dmg_path: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            check_app_update,
            check_core_update,
            get_app_version,
            get_core_version,
            download_app_update,
            install_app_update,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
