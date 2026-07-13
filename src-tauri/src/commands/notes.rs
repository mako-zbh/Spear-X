use crate::notes;

/// 获取工具笔记
#[tauri::command]
pub fn get_tool_note(tool_path: String, tool_name: String) -> Result<String, String> {
    notes::get_tool_note(&tool_path, &tool_name)
}

/// 保存工具笔记
#[tauri::command]
pub fn save_tool_note(tool_path: String, tool_name: String, content: String) -> Result<(), String> {
    notes::save_tool_note(&tool_path, &tool_name, &content)
}

/// 删除工具笔记
#[tauri::command]
pub fn delete_tool_note(tool_path: String, tool_name: String) -> Result<(), String> {
    notes::delete_tool_note(&tool_path, &tool_name)
}
