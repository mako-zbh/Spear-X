use crate::maintenance;

#[tauri::command]
pub fn cleanup_tool_paths_cmd() -> Result<(), String> {
    maintenance::cleanup_tool_paths()
}

#[tauri::command]
pub fn repair_config_file_cmd() -> Result<(), String> {
    maintenance::repair_config_file()
}

#[tauri::command]
pub fn cleanup_duplicate_tools_cmd() -> Result<(), String> {
    maintenance::cleanup_duplicate_tools()
}

#[tauri::command]
pub fn debug_all_tool_paths_cmd() -> Result<(), String> {
    maintenance::debug_all_tool_paths()
}
