use crate::config;
use crate::models::*;
use crate::notes;
use crate::paths;
use crate::scanner;
use tauri::{AppHandle, Emitter};

/// 添加新工具
#[tauri::command]
pub fn add_tool(app: AppHandle, tool: Tool, category_name: String) -> Result<(), String> {
    let (config_yaml, mut categories) = if paths::get_config_path().exists() {
        config::load_config()?
    } else {
        let config_yaml = ConfigYaml {
            java_paths: JavaConfig {
                java8: "resources/java8/bin/java".to_string(),
                java11: "resources/java11/bin/java".to_string(),
                java17: "resources/java17/bin/java".to_string(),
            },
            categories: vec![],
        };
        let categories = Categories::default();
        (config_yaml, categories)
    };

    // 检查工具名称是否已存在
    for category in &categories.categories {
        for existing_tool in &category.tools {
            if existing_tool.name == tool.name {
                return Err(format!("工具名称 '{}' 已存在", tool.name));
            }
        }
    }

    // 查找分类并添加工具
    let mut category_found = false;
    for category in &mut categories.categories {
        if category.name == category_name {
            category.tools.push(tool.clone());
            category_found = true;
            break;
        }
    }

    if !category_found {
        // 如果分类不存在，创建新分类
        let new_category = Category {
            name: category_name,
            icon: None,
            tools: vec![tool],
        };
        categories.categories.push(new_category);
    }

    config::save_categories_to_file(&categories, &config_yaml)?;

    // 发送更新成功事件
    let _ = app.emit("tool-added", true);
    Ok(())
}

/// 删除工具
#[tauri::command]
pub fn delete_tool(tool_name: String, category_name: String) -> Result<(), String> {
    let (config_yaml, mut categories) = config::load_config()?;

    let mut tool_found = false;
    for category in &mut categories.categories {
        if category.name == category_name {
            let original_len = category.tools.len();
            category.tools.retain(|t| t.name != tool_name);
            if category.tools.len() < original_len {
                tool_found = true;
                break;
            }
        }
    }

    if !tool_found {
        return Err(format!("未找到工具: {}", tool_name));
    }

    config::save_categories_to_file(&categories, &config_yaml)
}

/// 更新工具信息
#[tauri::command]
pub fn update_tool(
    app: AppHandle,
    original_name: String,
    category_name: String,
    tool: Tool,
) -> Result<(), String> {
    let (config_yaml, mut categories) = config::load_config()?;

    let mut found = false;
    let mut original_tool: Option<Tool> = None;

    for i in 0..categories.categories.len() {
        for j in 0..categories.categories[i].tools.len() {
            if categories.categories[i].tools[j].name == original_name {
                original_tool = Some(categories.categories[i].tools[j].clone());
                if categories.categories[i].name == category_name {
                    // 如果在同一分类中，直接更新工具
                    found = true;
                    categories.categories[i].tools[j] = tool.clone();
                    break;
                } else {
                    // 如果在不同分类中，从原分类删除
                    categories.categories[i].tools.remove(j);
                }
            }
        }
        if found {
            break;
        }
    }

    // 如果没有在原分类中找到或需要移动到新分类，则添加到目标分类
    if !found {
        for category in &mut categories.categories {
            if category.name == category_name {
                category.tools.push(tool.clone());
                break;
            }
        }
    }

    // 如果工具名称发生了变化，需要重命名对应的笔记文件
    if found && original_name != tool.name {
        if let Some(orig) = &original_tool {
            if !orig.path.is_empty() {
                if let Err(e) = notes::rename_tool_note(&orig.path, &original_name, &tool.name) {
                    eprintln!("重命名笔记文件失败: {}", e);
                }
            }
        }
    }

    config::save_categories_to_file(&categories, &config_yaml)?;

    let _ = app.emit("tool-updated", true);
    Ok(())
}

/// 更新工具描述
#[tauri::command]
pub fn update_tool_description(
    app: AppHandle,
    tool_name: String,
    category_name: String,
    description: String,
) -> Result<(), String> {
    let (config_yaml, mut categories) = config::load_config()?;

    let mut tool_found = false;
    for category in &mut categories.categories {
        if category.name == category_name {
            for tool in &mut category.tools {
                if tool.name == tool_name {
                    tool.description = Some(description.clone());
                    tool_found = true;
                    break;
                }
            }
            if tool_found {
                break;
            }
        }
    }

    if !tool_found {
        return Err(format!("未找到工具: {}", tool_name));
    }

    config::save_categories_to_file(&categories, &config_yaml)?;
    let _ = app.emit("tool-updated", true);
    Ok(())
}

/// 搜索工具（支持标签搜索）
#[tauri::command]
pub fn search_tools(query: String) -> Result<Vec<Tool>, String> {
    let categories = config::get_categories()?;

    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    // 检查是否是标签搜索
    let is_tag_search = query_lower.starts_with("标签:");
    if is_tag_search {
        let tag_query = query_lower["标签:".len()..].trim().to_string();

        for category in &categories.categories {
            for tool in &category.tools {
                for tag in &tool.tags {
                    if tag.to_lowercase().contains(&tag_query) {
                        results.push(tool.clone());
                        break;
                    }
                }
            }
        }
    } else {
        // 普通搜索
        for category in &categories.categories {
            for tool in &category.tools {
                let desc = tool.description.clone().unwrap_or_default();
                let source_url = tool.source_url.clone().unwrap_or_default();
                if tool.name.to_lowercase().contains(&query_lower)
                    || desc.to_lowercase().contains(&query_lower)
                    || tool.path.to_lowercase().contains(&query_lower)
                    || source_url.to_lowercase().contains(&query_lower)
                {
                    results.push(tool.clone());
                }
            }
        }
    }

    Ok(results)
}

/// 获取所有标签
#[tauri::command]
pub fn get_all_tags() -> Result<Vec<String>, String> {
    let categories = config::get_categories()?;

    let mut tag_set = std::collections::HashSet::new();

    for category in &categories.categories {
        for tool in &category.tools {
            for tag in &tool.tags {
                tag_set.insert(tag.clone());
            }
        }
    }

    Ok(tag_set.into_iter().collect())
}

/// 获取支持的工具类型
#[tauri::command]
pub fn get_tool_types() -> Vec<String> {
    vec![
        "Java8".to_string(),
        "Java11".to_string(),
        "Java17".to_string(),
        "Open".to_string(),
        "openterm".to_string(),
        "Browser".to_string(),
        "Binary".to_string(),
    ]
}

/// 获取工具的绝对路径
#[tauri::command]
pub fn get_tool_absolute_path(tool_path: String, file_name: String) -> Result<String, String> {
    config::get_tool_absolute_path(&tool_path, &file_name)
}

/// 获取真正的新工具（过滤掉已存在的）
#[tauri::command]
pub fn get_new_tools_from_scanned(tools: Vec<ScannedTool>) -> Result<Vec<ScannedTool>, String> {
    scanner::get_new_tools_from_scanned(&tools)
}

/// 自动添加扫描到的工具
#[tauri::command]
pub fn auto_add_scanned_tools(tools: Vec<ScannedTool>) -> Result<(), String> {
    scanner::auto_add_scanned_tools(&tools)
}
