use crate::config;
use crate::models::*;

/// 获取所有工具分类
#[tauri::command]
pub fn get_categories() -> Result<Categories, String> {
    config::get_categories()
}

/// 获取Java配置
#[tauri::command]
pub fn get_java_config() -> Result<Option<JavaConfig>, String> {
    let (config_yaml, _categories) = config::load_config()?;
    Ok(Some(config_yaml.java_paths))
}

/// 保存Java配置
#[tauri::command]
pub fn save_java_config(config_data: JavaConfig) -> Result<(), String> {
    let (mut config_yaml, categories) = config::load_config()?;
    config_yaml.java_paths = config_data;

    // 写回配置文件 — 使用 save_categories_to_file 保存
    config::save_categories_to_file(&categories, &config_yaml)
}

// Tauri v2 自动将 JS camelCase 参数名转换为 Rust snake_case：
// 前端 invoke('save_java_config', { configData: config })
//   → Rust save_java_config(config_data: JavaConfig)
// configData (camelCase) → config_data (snake_case) ✓
