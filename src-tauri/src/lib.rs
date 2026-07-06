mod update;
mod version;

use tauri::State;
use std::sync::Mutex;

/// 应用状态：easytier-core 当前版本号
struct AppState {
    core_version: Mutex<String>,
}

/// 问候测试命令（开发阶段用）
#[tauri::command]
fn greet(name: &str) -> String {
    format!("你好，{}！来自 Rust 的问候！", name)
}

/// 检查 App 自身更新
/// 从更新服务器获取最新版本信息
#[tauri::command]
fn check_app_update(_state: State<AppState>) -> Result<update::AppUpdateInfo, String> {
    let version = version::APP_VERSION;
    update::check_app_update(version).map_err(|e| format!("检查更新失败：{}", e))
}

/// 检查 easytier-core 更新
/// 从 GitHub Releases API 获取最新 core 版本
#[tauri::command]
fn check_core_update(state: State<AppState>) -> Result<update::CoreUpdateInfo, String> {
    let core_ver = state.core_version.lock().unwrap().clone();
    let ver = if core_ver.is_empty() {
        // 默认版本号（后续从实际安装的 core 获取）
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            core_version: Mutex::new(String::new()),
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            check_app_update,
            check_core_update,
            get_app_version,
            get_core_version,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
