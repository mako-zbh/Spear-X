use crate::maintenance;
use crate::models::*;
use crate::paths::get_resource_path;
use crate::scanner;
use std::path::Path;

/// 扫描resources文件夹寻找工具
#[tauri::command]
pub fn scan_resources_for_tools() -> Result<Vec<ScannedTool>, String> {
    let resources_path = Path::new(&get_resource_path()).join("resources");

    // 先清理无效的工具路径
    if let Err(e) = maintenance::clean_invalid_tool_paths() {
        eprintln!("清理无效工具路径时出错: {}", e);
        // 不返回错误，继续扫描
    }

    scanner::scan_tools_in_path(&resources_path.to_string_lossy())
}

/// 扫描自定义目录寻找工具
#[tauri::command]
pub fn scan_custom_directory_for_tools(custom_path: String) -> Result<Vec<ScannedTool>, String> {
    if !Path::new(&custom_path).exists() {
        return Err(format!("扫描目录不存在: {}", custom_path));
    }

    scanner::scan_tools_in_custom_path(&custom_path)
}

/// 扫描指定路径下的工具
#[tauri::command]
pub fn scan_tools_in_path(scan_path: String) -> Result<Vec<ScannedTool>, String> {
    scanner::scan_tools_in_path(&scan_path)
}

/// 扫描自定义路径下的工具（使用绝对路径）
#[tauri::command]
pub fn scan_tools_in_custom_path(scan_path: String) -> Result<Vec<ScannedTool>, String> {
    scanner::scan_tools_in_custom_path(&scan_path)
}

/// 清理无效路径并返回清理结果
#[tauri::command]
pub fn clean_invalid_paths() -> Result<CleanupResult, String> {
    let resources_path = Path::new(&get_resource_path()).join("resources");
    let scanned_tools = scanner::scan_tools_in_path(&resources_path.to_string_lossy())?;

    maintenance::clean_invalid_tool_paths_with_migration(&scanned_tools)
}
