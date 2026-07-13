use crate::executor;
use crate::models::Tool;

/// 执行工具命令（兼容旧版本）
#[tauri::command]
pub fn execute_command(
    path: String,
    optional: String,
    value: String,
    filename: String,
) -> Result<(), String> {
    executor::execute_command_with_custom(&path, &optional, &value, &filename, "", "")
}

/// 执行自定义命令（兼容旧版本）
#[tauri::command]
pub fn execute_custom_command(
    path: String,
    optional: String,
    value: String,
    filename: String,
    custom_command: String,
) -> Result<(), String> {
    executor::execute_command_with_custom(&path, &optional, &value, &filename, &custom_command, "")
}

/// 执行工具命令（支持自定义命令）
#[tauri::command]
pub fn execute_command_with_custom(
    path: String,
    optional: String,
    value: String,
    filename: String,
    custom_command: String,
    java_path: String,
) -> Result<(), String> {
    executor::execute_command_with_custom(
        &path,
        &optional,
        &value,
        &filename,
        &custom_command,
        &java_path,
    )
}

/// 执行工具命令（新版本，支持工具对象）
#[tauri::command]
pub fn execute_tool_command(tool: Tool, custom_command: String) -> Result<(), String> {
    executor::execute_command_with_custom(
        &tool.path,
        &tool.optional,
        &tool.value,
        &tool.file_name,
        &custom_command,
        "",
    )
}
