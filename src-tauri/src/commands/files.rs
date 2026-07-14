use crate::config;
use crate::models::*;
use crate::paths::{self, get_resource_path};
use crate::scanner;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// 浏览指定目录（支持相对路径和绝对路径）
#[tauri::command]
pub fn browse_directory(path_input: String) -> Result<Vec<FileInfo>, String> {
    let mut file_infos = Vec::new();

    println!("BrowseDirectory 调用，输入路径: {}", path_input);

    let full_path = if Path::new(&path_input).is_absolute() {
        // 绝对路径，直接使用
        println!("使用绝对路径: {}", path_input);
        Path::new(&path_input).to_path_buf()
    } else {
        // 相对路径，构建基于resources的完整路径
        let base_path = get_resource_path();
        if path_input.is_empty() || path_input == "/" {
            // 浏览resources根目录
            let p = Path::new(&base_path).join("resources");
            println!("浏览根目录，完整路径: {}", p.display());
            p
        } else {
            let p = Path::new(&base_path).join("resources").join(&path_input);
            println!("浏览子目录，完整路径: {}", p.display());
            p
        }
    };

    // 检查路径是否存在
    if !full_path.exists() {
        return Err(format!(
            "目录不存在: {} (完整路径: {})",
            path_input,
            full_path.display()
        ));
    }

    // 读取目录内容
    let files = fs::read_dir(&full_path)
        .map_err(|e| format!("读取目录失败: {}", e))?;

    for file in files.flatten() {
        let file_name = file.file_name().to_string_lossy().to_string();

        // 跳过隐藏文件
        if file_name.starts_with('.') {
            continue;
        }

        let file_ext = Path::new(&file_name)
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
            .unwrap_or_default();

        let metadata = file.metadata().ok();
        let is_dir = file
            .file_type()
            .map(|t| t.is_dir())
            .unwrap_or(false);

        let is_executable = if let Some(ref meta) = metadata {
            scanner::is_executable_file(&file_name, is_dir, Some(meta))
        } else {
            false
        };

        let file_path = if path_input.is_empty() || path_input == "/" {
            file_name.clone()
        } else {
            format!("{}/{}", path_input, file_name)
        };

        let mod_time = if let Some(ref meta) = metadata {
            if let Ok(modified) = meta.modified() {
                if let Ok(datetime) = modified.duration_since(std::time::UNIX_EPOCH) {
                    format!("{}", datetime.as_secs())
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let size = metadata.as_ref().map(|m| m.len() as i64).unwrap_or(0);

        let file_info = FileInfo {
            name: file_name,
            is_dir,
            size,
            mod_time,
            path: file_path,
            extension: file_ext,
            is_executable,
        };

        file_infos.push(file_info);
    }

    Ok(file_infos)
}

/// 获取工具目录的文件列表（用于编辑工具时选择文件）
#[tauri::command]
pub fn get_tool_directory(tool_path: String) -> Result<Vec<FileInfo>, String> {
    println!("GetToolDirectory 调用，原始路径: {}", tool_path);

    // 空路径校验
    if tool_path.trim().is_empty() {
        return Err("工具路径不能为空".to_string());
    }

    // 绝对路径 / URL：直接交给 browse_directory，它本身支持绝对路径。
    // （URL 型工具不会走到这里，前端 loadFileBrowser 已拦截 Browser 类型）
    if paths::is_url(&tool_path) {
        return Ok(vec![]);
    }
    if Path::new(&tool_path).is_absolute() {
        return browse_directory(tool_path);
    }

    // 相对路径：清理后去掉 resources/ 前缀，因为 BrowseDirectory 会自动拼回 resources/
    let cleaned_path = config::clean_tool_path(&tool_path);
    let clean_path = cleaned_path.strip_prefix("resources/").unwrap_or("");

    // 清理后为空，说明原本就只填了 "resources/" 这种无意义路径
    if clean_path.is_empty() {
        return Err("工具路径不能为空".to_string());
    }

    println!("最终使用的相对路径: {}", clean_path);

    browse_directory(clean_path.to_string())
}

/// 获取文件信息
#[tauri::command]
pub fn get_file_info(file_path: String) -> Result<HashMap<String, String>, String> {
    let abs_path = std::path::Path::new(&file_path)
        .canonicalize()
        .map_err(|e| format!("无法获取绝对路径: {}", e))?;

    let dir = abs_path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let file_name = abs_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let abs_str = abs_path.to_string_lossy().to_string();

    let mut map = HashMap::new();
    map.insert("path".to_string(), dir);
    map.insert("fileName".to_string(), file_name);
    map.insert("fullPath".to_string(), abs_str);

    Ok(map)
}

/// 获取文件的完整路径
#[tauri::command]
pub fn get_file_path(file_name: String) -> Result<String, String> {
    let abs_path = std::path::Path::new(&file_name)
        .canonicalize()
        .or_else(|_| {
            if std::path::Path::new(&file_name).is_absolute() {
                Ok(std::path::PathBuf::from(&file_name))
            } else {
                std::env::current_dir().map(|d| d.join(&file_name))
            }
        })
        .map_err(|e| format!("获取绝对路径失败: {}", e))?;

    Ok(abs_path.to_string_lossy().to_string())
}
