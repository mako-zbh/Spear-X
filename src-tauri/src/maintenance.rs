use crate::config;
use crate::models::*;
use crate::notes;
use crate::paths;
use std::fs;
use std::path::Path;

/// 修复损坏的配置文件
pub fn repair_config_file() -> Result<(), String> {
    let config_path = paths::get_config_path();

    // 首先验证当前配置文件是否正常
    if config::validate_config_file(&config_path).is_ok() {
        // 配置文件正常，无需修复
        return Ok(());
    }

    println!("检测到配置文件异常，开始修复...");
    let backup_path = format!("{}.backup", config_path.to_string_lossy());

    // 检查是否有备份文件
    if Path::new(&backup_path).exists() {
        println!("发现备份文件: {}", backup_path);

        // 验证备份文件
        if config::validate_config_file(Path::new(&backup_path)).is_ok() {
            println!("备份文件验证通过，开始恢复...");

            // 删除损坏的文件
            let _ = fs::remove_file(&config_path);

            // 恢复备份
            fs::rename(&backup_path, &config_path).map_err(|e| format!("恢复备份失败: {}", e))?;

            println!("配置文件修复成功！");
            return Ok(());
        } else {
            println!("备份文件也已损坏");
        }
    }

    // 如果没有可用备份，创建默认配置
    println!("没有可用备份，创建默认配置...");
    let default_config = ConfigYaml {
        java_paths: JavaConfig {
            java8: String::new(),
            java11: String::new(),
            java17: String::new(),
        },
        categories: vec![],
    };

    let yaml_data = serde_yaml::to_string(&default_config)
        .map_err(|e| format!("序列化默认配置失败: {}", e))?;

    let content = format!(
        "# Java配置\n# 自定义Java路径配置，如果留空将使用系统默认Java\n{}",
        yaml_data
    );

    fs::write(&config_path, &content).map_err(|e| format!("写入默认配置失败: {}", e))?;

    println!("已创建默认配置文件");
    Ok(())
}

/// 清理和修复工具路径
pub fn cleanup_tool_paths() -> Result<(), String> {
    let (config_yaml, categories) = config::load_config()?;

    // 清理每个工具的路径
    let mut paths_fixed = 0;
    let mut categories = categories;

    for category in &mut categories.categories {
        for tool in &mut category.tools {
            let original_path = tool.path.clone();
            let cleaned_path = config::clean_tool_path(&tool.path);

            if original_path != cleaned_path {
                println!("修复工具路径: {} -> {}", original_path, cleaned_path);
                tool.path = cleaned_path;
                paths_fixed += 1;
            }
        }
    }

    // 如果有路径被修复，保存配置
    if paths_fixed > 0 {
        println!("总共修复了 {} 个工具路径", paths_fixed);
        return config::save_categories_to_file(&categories, &config_yaml);
    }

    println!("没有发现需要修复的路径");
    Ok(())
}

/// 清理重复的工具
pub fn cleanup_duplicate_tools() -> Result<(), String> {
    let (config_yaml, categories) = config::load_config()?;
    let mut categories = categories;

    println!("开始清理重复工具...");

    // 记录已处理的工具路径和对应的分类映射
    let mut processed_paths: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut duplicates_found = 0;

    // 第一轮：找出最佳分类（优先选择中文分类名）
    for category in &categories.categories {
        for tool in &category.tools {
            if let Some(existing_category) = processed_paths.get(&tool.path) {
                // 发现重复工具
                duplicates_found += 1;
                println!("发现重复工具: {}", tool.path);
                println!("  已存在分类: {}", existing_category);
                println!("  当前分类: {}", category.name);

                // 选择更好的分类名
                if is_better_category_name(&category.name, existing_category) {
                    processed_paths.insert(tool.path.clone(), category.name.clone());
                    println!("  选择分类: {}", category.name);
                } else {
                    println!("  保持分类: {}", existing_category);
                }
            } else {
                processed_paths.insert(tool.path.clone(), category.name.clone());
            }
        }
    }

    if duplicates_found == 0 {
        println!("没有发现重复工具");
        return Ok(());
    }

    println!("发现 {} 个重复工具，开始合并...", duplicates_found);

    // 第二轮：重建分类，合并重复工具
    let mut new_categories: Vec<Category> = Vec::new();
    let mut category_map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for category in &categories.categories {
        for tool in &category.tools {
            let best_category_name = processed_paths.get(&tool.path).cloned().unwrap_or_default();

            if let Some(&idx) = category_map.get(&best_category_name) {
                // 检查工具是否已经存在于目标分类中
                let mut tool_exists = false;
                for existing_tool in &mut new_categories[idx].tools {
                    if existing_tool.path == tool.path {
                        tool_exists = true;
                        // 如果新工具有文件名而现有工具没有，则更新
                        if existing_tool.file_name.is_empty() && !tool.file_name.is_empty() {
                            existing_tool.file_name = tool.file_name.clone();
                            existing_tool.value = tool.value.clone();
                            existing_tool.command = tool.command.clone();
                            println!("更新工具文件名: {} -> {}", tool.path, tool.file_name);
                        }
                        break;
                    }
                }

                if !tool_exists {
                    new_categories[idx].tools.push(tool.clone());
                }
            } else {
                // 创建新分类
                let new_category = Category {
                    name: best_category_name.clone(),
                    icon: category.icon.clone(),
                    tools: vec![tool.clone()],
                };
                let idx = new_categories.len();
                new_categories.push(new_category);
                category_map.insert(best_category_name, idx);
            }
        }
    }

    categories.categories = new_categories;

    config::save_categories_to_file(&categories, &config_yaml)?;

    println!("重复工具清理完成，合并了 {} 个重复工具", duplicates_found);
    Ok(())
}

/// 判断哪个分类名更好（中文优先，然后选更长的）
fn is_better_category_name(name1: &str, name2: &str) -> bool {
    if is_chinese(name1) && !is_chinese(name2) {
        return true;
    }
    if !is_chinese(name1) && is_chinese(name2) {
        return false;
    }
    name1.chars().count() > name2.chars().count()
}

/// 判断字符串是否包含中文字符
fn is_chinese(s: &str) -> bool {
    for r in s.chars() {
        if (r as u32) >= 0x4e00 && (r as u32) <= 0x9fff {
            return true;
        }
    }
    false
}

/// 清理配置中无效的工具路径
pub fn clean_invalid_tool_paths() -> Result<(), String> {
    let _ = clean_invalid_tool_paths_with_result()?;
    Ok(())
}

/// 清理配置中无效的工具路径并返回详细结果
pub fn clean_invalid_tool_paths_with_result() -> Result<CleanupResult, String> {
    let mut result = CleanupResult {
        invalid_tool_names: vec![],
        ..Default::default()
    };

    let (config_yaml, categories) = config::load_config()?;
    let base_path = paths::get_resource_path();
    let mut cleaned_categories = Categories::default();
    let original_category_count = categories.categories.len();

    for category in &categories.categories {
        let mut cleaned_category = Category {
            name: category.name.clone(),
            icon: category.icon.clone(),
            tools: vec![],
        };

        for tool in &category.tools {
            // 对于Browser类型的工具，如果路径是URL，跳过文件系统检查
            if tool.value == "Browser"
                && (tool.path.starts_with("http://") || tool.path.starts_with("https://"))
            {
                cleaned_category.tools.push(tool.clone());
                continue;
            }

            // 检查工具路径是否存在
            let full_tool_path = if Path::new(&tool.path).is_absolute() {
                if tool.file_name.is_empty() {
                    PathBuf::from(&tool.path)
                } else {
                    PathBuf::from(&tool.path).join(&tool.file_name)
                }
            } else {
                let base = PathBuf::from(&base_path).join(&tool.path);
                if tool.file_name.is_empty() {
                    base
                } else {
                    base.join(&tool.file_name)
                }
            };

            let path_exists = if !tool.file_name.is_empty()
                && tool.file_name.to_lowercase().ends_with(".app")
            {
                // .app文件是目录，检查目录是否存在
                if let Ok(meta) = fs::metadata(&full_tool_path) {
                    meta.is_dir()
                } else {
                    false
                }
            } else {
                full_tool_path.exists()
            };

            if !path_exists {
                println!(
                    "发现无效工具路径: {} (工具: {}, FileName: {})",
                    tool.path, tool.name, tool.file_name
                );
                result.invalid_tools_count += 1;
                result.invalid_tool_names.push(tool.name.clone());

                // 删除对应的笔记文件
                if notes::clean_tool_note(&tool.path) {
                    result.cleaned_notes += 1;
                }
                continue;
            }

            cleaned_category.tools.push(tool.clone());
        }

        if !cleaned_category.tools.is_empty() {
            cleaned_categories.categories.push(cleaned_category);
        } else if !category.tools.is_empty() {
            println!("分类 '{}' 的所有工具都无效，已删除该分类", category.name);
        }
    }

    result.invalid_categories_count = (original_category_count - cleaned_categories.categories.len()) as i32;

    if result.invalid_tools_count > 0 {
        config::save_categories_to_file(&cleaned_categories, &config_yaml)?;
        println!(
            "已清理 {} 个无效工具路径，{} 个无效分类，{} 个笔记文件",
            result.invalid_tools_count, result.invalid_categories_count, result.cleaned_notes
        );
    }

    Ok(result)
}

/// 清理配置中无效的工具路径，支持智能迁移检测
pub fn clean_invalid_tool_paths_with_migration(
    scanned_tools: &[ScannedTool],
) -> Result<CleanupResult, String> {
    let mut result = CleanupResult {
        invalid_tool_names: vec![],
        migrated_tool_names: vec![],
        ..Default::default()
    };

    let (config_yaml, categories) = config::load_config()?;

    // 创建扫描到的工具目录名映射，用于迁移检测
    let mut scanned_tool_dirs: std::collections::HashMap<String, &ScannedTool> =
        std::collections::HashMap::new();
    for scanned_tool in scanned_tools {
        let path_parts: Vec<&str> = scanned_tool.path.split('/').collect();
        if !path_parts.is_empty() {
            let tool_dir_name = path_parts[path_parts.len() - 1].to_string();
            scanned_tool_dirs.insert(tool_dir_name, scanned_tool);
        }
    }

    let base_path = paths::get_resource_path();
    let mut cleaned_categories = Categories::default();
    let original_category_count = categories.categories.len();

    for category in &categories.categories {
        let mut cleaned_category = Category {
            name: category.name.clone(),
            icon: category.icon.clone(),
            tools: vec![],
        };

        for tool in &category.tools {
            // 对于Browser类型的工具，如果路径是URL，跳过文件系统检查
            if tool.value == "Browser"
                && (tool.path.starts_with("http://") || tool.path.starts_with("https://"))
            {
                cleaned_category.tools.push(tool.clone());
                continue;
            }

            // 检查工具路径是否存在
            let full_tool_path = if Path::new(&tool.path).is_absolute() {
                if tool.file_name.is_empty() {
                    PathBuf::from(&tool.path)
                } else {
                    PathBuf::from(&tool.path).join(&tool.file_name)
                }
            } else {
                let base = PathBuf::from(&base_path).join(&tool.path);
                if tool.file_name.is_empty() {
                    base
                } else {
                    base.join(&tool.file_name)
                }
            };

            let path_exists = if !tool.file_name.is_empty()
                && tool.file_name.to_lowercase().ends_with(".app")
            {
                if let Ok(meta) = fs::metadata(&full_tool_path) {
                    meta.is_dir()
                } else {
                    false
                }
            } else {
                full_tool_path.exists()
            };

            if !path_exists {
                // 对于绝对路径的工具，直接标记为无效
                if Path::new(&tool.path).is_absolute() {
                    println!(
                        "发现无效工具路径（绝对路径）: {} (工具: {})",
                        tool.path, tool.name
                    );
                    result.invalid_tools_count += 1;
                    result.invalid_tool_names.push(tool.name.clone());

                    if notes::clean_tool_note(&tool.path) {
                        result.cleaned_notes += 1;
                    }
                    continue;
                }

                // 对于相对路径，检查是否有迁移的可能
                let path_parts: Vec<&str> = tool.path.split('/').collect();
                if !path_parts.is_empty() {
                    let tool_dir_name = path_parts[path_parts.len() - 1];

                    if let Some(new_scanned_tool) = scanned_tool_dirs.get(tool_dir_name) {
                        // 发现可能的迁移，迁移笔记而不是删除
                        println!(
                            "检测到工具迁移: {} ({} -> {})",
                            tool.name, tool.path, new_scanned_tool.path
                        );

                        if notes::migrate_tool_note(tool_dir_name) {
                            result.migrated_notes += 1;
                            result.migrated_tool_names.push(tool.name.clone());
                            println!("已迁移工具笔记: {}", tool.name);
                        }

                        result.invalid_tools_count += 1;
                        result.invalid_tool_names.push(tool.name.clone());
                        continue;
                    }
                }

                // 没有找到迁移目标，按原逻辑处理
                println!(
                    "发现无效工具路径: {} (工具: {}, FileName: {})",
                    tool.path, tool.name, tool.file_name
                );
                result.invalid_tools_count += 1;
                result.invalid_tool_names.push(tool.name.clone());

                if notes::clean_tool_note(&tool.path) {
                    result.cleaned_notes += 1;
                }
                continue;
            }

            cleaned_category.tools.push(tool.clone());
        }

        if !cleaned_category.tools.is_empty() {
            cleaned_categories.categories.push(cleaned_category);
        } else if !category.tools.is_empty() {
            println!("分类 '{}' 的所有工具都无效，已删除该分类", category.name);
        }
    }

    result.invalid_categories_count = (original_category_count - cleaned_categories.categories.len()) as i32;

    if result.invalid_tools_count > 0 {
        config::save_categories_to_file(&cleaned_categories, &config_yaml)?;
        println!(
            "已清理 {} 个无效工具路径，{} 个无效分类，{} 个笔记文件，迁移 {} 个笔记文件",
            result.invalid_tools_count,
            result.invalid_categories_count,
            result.cleaned_notes,
            result.migrated_notes
        );
    }

    Ok(result)
}

/// 调试方法：打印所有工具的路径配置
pub fn debug_all_tool_paths() -> Result<(), String> {
    let categories = config::get_categories()?;

    println!("=== 调试：所有工具路径配置 ===");
    for category in &categories.categories {
        println!("分类: {}", category.name);
        for tool in &category.tools {
            println!("  工具: {}, 路径: {}", tool.name, tool.path);
        }
    }
    println!("=== 调试结束 ===");
    Ok(())
}

use std::path::PathBuf;
