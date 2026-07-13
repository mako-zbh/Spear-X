use std::collections::HashMap;
use tauri_plugin_dialog::DialogExt;

/// 打开文件选择对话框
#[tauri::command]
pub async fn open_file_dialog(app: tauri::AppHandle) -> Result<Option<HashMap<String, String>>, String> {
    let file_path = app
        .dialog()
        .file()
        .set_title("选择工具文件")
        .blocking_pick_file();

    match file_path {
        Some(path) => {
            let path_str = path.to_string();
            let abs_path = std::path::Path::new(&path_str)
                .canonicalize()
                .map_err(|e| format!("无法获取绝对路径: {}", e))?;
            let abs_str = abs_path.to_string_lossy().to_string();

            let dir = abs_path
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            let file_name = abs_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let mut map = HashMap::new();
            map.insert("path".to_string(), dir);
            map.insert("fileName".to_string(), file_name);
            map.insert("fullPath".to_string(), abs_str);

            Ok(Some(map))
        }
        None => Ok(None),
    }
}

/// 打开目录选择对话框
#[tauri::command]
pub async fn open_directory_dialog(app: tauri::AppHandle) -> Result<String, String> {
    let dir_path = app
        .dialog()
        .file()
        .set_title("选择工具目录")
        .blocking_pick_folder();

    match dir_path {
        Some(path) => Ok(path.to_string()),
        None => Ok(String::new()),
    }
}

/// 选择目录（用于前端文件夹选择器）
#[tauri::command]
pub async fn select_directory(app: tauri::AppHandle) -> Result<String, String> {
    let selected_path = app
        .dialog()
        .file()
        .set_title("选择要扫描的文件夹")
        .blocking_pick_folder();

    match selected_path {
        Some(path) => Ok(path.to_string()),
        None => Ok(String::new()),
    }
}

/// 选择文件
#[tauri::command]
pub async fn select_file(app: tauri::AppHandle) -> Result<String, String> {
    let selected_file = app
        .dialog()
        .file()
        .set_title("选择文件")
        .blocking_pick_file();

    match selected_file {
        Some(path) => Ok(path.to_string()),
        None => Ok(String::new()),
    }
}

/// 选择Java路径（选择具体的Java可执行文件）
#[tauri::command]
pub async fn select_java_path(app: tauri::AppHandle) -> Result<String, String> {
    let selected_file = app
        .dialog()
        .file()
        .set_title("选择Java可执行文件")
        .blocking_pick_file();

    match selected_file {
        Some(path) => Ok(path.to_string()),
        None => Ok(String::new()),
    }
}

/// 打开文件/目录选择对话框（限制在 App 包内 resources 目录下）
#[tauri::command]
pub async fn select(app: tauri::AppHandle, select_folder: bool) -> Result<String, String> {
    let resources_dir = "/Applications/Spear.app/Contents/Resources";

    let result = if select_folder {
        app.dialog()
            .file()
            .set_title("选择工具")
            .set_directory(resources_dir)
            .blocking_pick_folder()
    } else {
        app.dialog()
            .file()
            .set_title("选择工具")
            .set_directory(resources_dir)
            .blocking_pick_file()
    };

    match result {
        Some(path) => {
            let dialog = path.to_string();
            // 验证路径
            if !dialog.contains("Contents/Resources") {
                return Err("无效的工具路径：必须位于 App包内 resources 目录下".to_string());
            }
            // 提取相对路径
            let parts: Vec<&str> = dialog.split("Contents/Resources/").collect();
            if parts.len() != 2 {
                return Err("无法解析工具路径".to_string());
            }
            Ok(parts[1].to_string())
        }
        None => Ok(String::new()),
    }
}
