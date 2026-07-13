use std::path::{Path, PathBuf};

/// 获取用户配置目录（应用包外，不受更新覆盖影响）
/// macOS: ~/Library/Application Support/SpearX
/// Windows: %APPDATA%/SpearX
/// Linux: ~/.config/spearx
pub fn get_config_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            return home.join("Library").join("Application Support").join("SpearX");
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = dirs::config_dir() {
            return appdata.join("SpearX");
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(config) = dirs::config_dir() {
            return config.join("spearx");
        }
    }
    #[allow(unreachable_code)]
    PathBuf::from(".")
}

/// 获取用户配置文件的完整路径
pub fn get_config_path() -> PathBuf {
    get_config_dir().join("tool.yml")
}

/// 获取笔记目录路径（位于配置目录下）
pub fn get_notes_dir() -> PathBuf {
    get_config_dir().join("notes")
}

/// 获取资源路径
/// 对于 .app 包，可执行文件在 Contents/MacOS 目录下，资源文件在 Contents/Resources 目录下
pub fn get_resource_path() -> String {
    if let Ok(exec_path) = std::env::current_exe() {
        let exec_str = exec_path.to_string_lossy().to_string();

        // 通用检测：如果路径包含 Contents/MacOS，则认为是 .app 包
        if exec_str.contains("/Contents/MacOS/") {
            if let Some(parent) = exec_path.parent() {
                return parent
                    .join("../Resources")
                    .to_string_lossy()
                    .to_string();
            }
        }

        // 在开发模式下，如果路径包含 build/bin，则使用 .app 包内的 Resources 目录
        if exec_str.contains("build/bin") {
            if let Some(parent) = exec_path.parent() {
                let app_resources_path = parent.join("../Resources");
                if let Ok(abs_path) = app_resources_path.canonicalize() {
                    return abs_path.to_string_lossy().to_string();
                }

                // 尝试从项目根目录找到 build/bin/SpearX.app/Contents/Resources
                let project_root = parent.join("../../../../../");
                if let Ok(abs_project_root) = project_root.canonicalize() {
                    let app_resources_path = abs_project_root
                        .join("build/bin/SpearX.app/Contents/Resources");
                    if app_resources_path.exists() {
                        return app_resources_path.to_string_lossy().to_string();
                    }
                }
            }
        }

        // 默认返回可执行文件所在目录
        if let Some(parent) = exec_path.parent() {
            return parent.to_string_lossy().to_string();
        }
    }
    ".".to_string()
}

/// 判断是否为绝对路径
pub fn is_absolute_path(path: &str) -> bool {
    Path::new(path).is_absolute()
}

/// 判断是否为 URL
pub fn is_url(path: &str) -> bool {
    path.starts_with("http://") || path.starts_with("https://")
}
