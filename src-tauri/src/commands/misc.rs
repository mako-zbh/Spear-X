use crate::platform;
use std::collections::HashMap;

/// 打开工具所在目录
#[tauri::command]
pub fn open_tool_directory(path: String) -> Result<(), String> {
    let full_path = if std::path::Path::new(&path).is_absolute() {
        path
    } else {
        let resource_path = crate::paths::get_resource_path();
        std::path::Path::new(&resource_path)
            .join(&path)
            .to_string_lossy()
            .to_string()
    };

    platform::open_in_file_manager(&full_path)
}

/// 在默认浏览器中打开GitHub页面
#[tauri::command]
pub fn open_github_page() -> Result<(), String> {
    let github_url = "https://github.com/mako-zbh/Spear-X";
    platform::open_url(github_url)
}

/// 获取预定义的执行方式列表
#[tauri::command]
pub fn get_file_types() -> Vec<HashMap<String, String>> {
    vec![
        {
            let mut m = HashMap::new();
            m.insert("value".to_string(), "Java8".to_string());
            m.insert("label".to_string(), "Java 8".to_string());
            m.insert("description".to_string(), "使用Java 8运行JAR文件".to_string());
            m
        },
        {
            let mut m = HashMap::new();
            m.insert("value".to_string(), "Java11".to_string());
            m.insert("label".to_string(), "Java 11".to_string());
            m.insert("description".to_string(), "使用Java 11运行JAR文件".to_string());
            m
        },
        {
            let mut m = HashMap::new();
            m.insert("value".to_string(), "Java17".to_string());
            m.insert("label".to_string(), "Java 17".to_string());
            m.insert("description".to_string(), "使用Java 17运行JAR文件".to_string());
            m
        },
        {
            let mut m = HashMap::new();
            m.insert("value".to_string(), "Open".to_string());
            m.insert("label".to_string(), "系统打开".to_string());
            m.insert("description".to_string(), "使用系统默认方式打开文件".to_string());
            m
        },
        {
            let mut m = HashMap::new();
            m.insert("value".to_string(), "openterm".to_string());
            m.insert("label".to_string(), "终端打开".to_string());
            m.insert("description".to_string(), "在终端中打开目录".to_string());
            m
        },
        {
            let mut m = HashMap::new();
            m.insert("value".to_string(), "python".to_string());
            m.insert("label".to_string(), "Python".to_string());
            m.insert("description".to_string(), "使用Python运行脚本".to_string());
            m
        },
        {
            let mut m = HashMap::new();
            m.insert("value".to_string(), "custom".to_string());
            m.insert("label".to_string(), "自定义命令".to_string());
            m.insert("description".to_string(), "使用自定义系统命令".to_string());
            m
        },
    ]
}
