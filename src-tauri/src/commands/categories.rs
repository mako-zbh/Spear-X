use crate::config;
use crate::models::*;
use tauri::{AppHandle, Emitter};

/// 添加新分类
#[tauri::command]
pub fn add_category(category_name: String) -> Result<(), String> {
    let (config_yaml, mut categories) = config::load_config()?;

    // 检查分类是否已存在
    for category in &categories.categories {
        if category.name == category_name {
            return Err(format!("分类 '{}' 已存在", category_name));
        }
    }

    // 添加新分类
    let new_category = Category {
        name: category_name,
        icon: None,
        tools: vec![],
    };
    categories.categories.push(new_category);

    config::save_categories_to_file(&categories, &config_yaml)
}

/// 删除分类及其下的所有工具
#[tauri::command]
pub fn delete_category(app: AppHandle, category_name: String) -> Result<(), String> {
    let (config_yaml, mut categories) = config::load_config()?;

    let original_len = categories.categories.len();
    categories.categories.retain(|c| c.name != category_name);

    if categories.categories.len() == original_len {
        return Err(format!("分类 '{}' 不存在", category_name));
    }

    config::save_categories_to_file(&categories, &config_yaml)?;
    let _ = app.emit("category-deleted", true);
    Ok(())
}

/// 批量更新分类下工具顺序
#[tauri::command]
pub fn update_category_tools(
    app: AppHandle,
    category_name: String,
    tools: Vec<Tool>,
) -> Result<(), String> {
    let (config_yaml, mut categories) = config::load_config()?;

    for category in &mut categories.categories {
        if category.name == category_name {
            category.tools = tools;
            break;
        }
    }

    config::save_categories_to_file(&categories, &config_yaml)?;
    let _ = app.emit("tool-updated", true);
    Ok(())
}

/// 更新分类名称
#[tauri::command]
pub fn update_category_name(old_name: String, new_name: String) -> Result<(), String> {
    let (config_yaml, mut categories) = config::load_config()?;

    let mut found = false;
    for category in &mut categories.categories {
        if category.name == old_name {
            category.name = new_name.clone();
            found = true;
            break;
        }
    }

    if !found {
        return Err(format!("分类 '{}' 不存在", old_name));
    }

    config::save_categories_to_file(&categories, &config_yaml)
}

/// 更新分类顺序
#[tauri::command]
pub fn update_categories_order(ordered_categories: Vec<Category>) -> Result<(), String> {
    let (config_yaml, _categories) = config::load_config()?;

    let categories = Categories {
        categories: ordered_categories,
    };

    config::save_categories_to_file(&categories, &config_yaml)
}

/// 更新分类图标
#[tauri::command]
pub fn update_category_icon(category_name: String, icon: String) -> Result<(), String> {
    let (config_yaml, mut categories) = config::load_config()?;

    let mut found = false;
    for category in &mut categories.categories {
        if category.name == category_name {
            category.icon = Some(icon.clone());
            found = true;
            break;
        }
    }

    if !found {
        return Err(format!("分类 '{}' 不存在", category_name));
    }

    config::save_categories_to_file(&categories, &config_yaml)
}
