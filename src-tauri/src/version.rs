// 应用版本信息
// 版本号以 VERSION 文件为准，构建时通过脚本注入

/// 当前应用版本号（与 VERSION 文件保持一致）
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
