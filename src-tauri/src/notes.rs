use crate::config;
use crate::paths;
use std::fs;
use std::path::Path;

/// 获取工具笔记 (新版本：从工具文件夹中读取)
pub fn get_tool_note(tool_path: &str, tool_name: &str) -> Result<String, String> {
    if tool_path.is_empty() {
        return Err("工具路径不能为空".to_string());
    }

    // 获取工具的绝对路径
    let tool_dir = config::get_tool_absolute_path(tool_path, "")?;
    let note_file = Path::new(&tool_dir).join(format!("{}.md", tool_name));

    // 如果文件不存在，尝试查找旧位置的笔记并迁移
    if !note_file.exists() {
        // 尝试从旧的notes目录查找并迁移
        if let Some(content) = find_and_migrate_old_note(tool_path, tool_name) {
            return Ok(content);
        }

        // 尝试查找同目录下的其他.md文件
        if let Some(content) = find_other_notes_in_tool_dir(&tool_dir, tool_name) {
            return Ok(content);
        }

        return Ok(String::new());
    }

    fs::read_to_string(&note_file).map_err(|e| format!("读取笔记失败: {}", e))
}

/// 保存工具笔记 (新版本：保存到工具文件夹中)
pub fn save_tool_note(tool_path: &str, tool_name: &str, content: &str) -> Result<(), String> {
    if tool_path.is_empty() {
        return Err("工具路径不能为空".to_string());
    }

    // 获取工具的绝对路径
    let tool_dir = config::get_tool_absolute_path(tool_path, "")?;

    // 确保工具目录存在
    fs::create_dir_all(&tool_dir).map_err(|e| format!("创建工具目录失败: {}", e))?;

    let note_file = Path::new(&tool_dir).join(format!("{}.md", tool_name));
    fs::write(&note_file, content).map_err(|e| format!("写入笔记失败: {}", e))
}

/// 删除工具笔记 (新版本：从工具文件夹中删除)
pub fn delete_tool_note(tool_path: &str, tool_name: &str) -> Result<(), String> {
    if tool_path.is_empty() {
        return Ok(()); // 路径为空，无需删除
    }

    // 获取工具的绝对路径
    let tool_dir = config::get_tool_absolute_path(tool_path, "")?;
    let note_file = Path::new(&tool_dir).join(format!("{}.md", tool_name));

    // 检查文件是否存在
    if !note_file.exists() {
        return Ok(()); // 文件不存在，不需要删除
    }

    fs::remove_file(&note_file).map_err(|e| format!("删除笔记失败: {}", e))
}

/// 查找并迁移旧位置的笔记
fn find_and_migrate_old_note(tool_path: &str, tool_name: &str) -> Option<String> {
    let notes_dir = paths::get_notes_dir();

    // 生成可能的旧笔记ID
    let path_parts: Vec<&str> = tool_path.split('/').collect();
    if path_parts.is_empty() {
        return None;
    }

    let tool_dir_name = path_parts[path_parts.len() - 1];
    let possible_ids = vec![
        tool_dir_name.to_string(),
        tool_dir_name.replace(' ', "_"),
        tool_dir_name.replace('-', "_"),
    ];

    for old_id in &possible_ids {
        let old_note_file = notes_dir.join(format!("{}.md", old_id));
        if let Ok(content) = fs::read_to_string(&old_note_file) {
            // 找到旧笔记，迁移到新位置
            if save_tool_note(tool_path, tool_name, &content).is_ok() {
                // 迁移成功，删除旧文件
                let _ = fs::remove_file(&old_note_file);
                println!(
                    "已迁移笔记: {} -> {}/{}.md",
                    old_note_file.display(),
                    tool_path,
                    tool_name
                );
                return Some(content);
            }
        }
    }

    None
}

/// 在工具目录中查找其他笔记文件
fn find_other_notes_in_tool_dir(tool_dir: &str, current_tool_name: &str) -> Option<String> {
    let files = match fs::read_dir(tool_dir) {
        Ok(f) => f,
        Err(_) => return None,
    };

    let expected_file_name = format!("{}.md", current_tool_name);

    for file in files.flatten() {
        let file_name = file.file_name().to_string_lossy().to_string();
        let is_dir = file.file_type().map(|t| t.is_dir()).unwrap_or(false);

        if !is_dir && file_name.ends_with(".md") {
            // 排除当前工具名称的笔记文件
            if file_name != expected_file_name {
                let note_file = file.path();
                if let Ok(content) = fs::read_to_string(&note_file) {
                    // 将找到的笔记迁移到正确的文件名
                    // 计算相对路径
                    let resource_path = paths::get_resource_path();
                    let tool_dir_path = Path::new(tool_dir);
                    let resource_dir = Path::new(&resource_path);
                    let relative_path = tool_dir_path
                        .strip_prefix(resource_dir)
                        .unwrap_or(Path::new(""))
                        .to_string_lossy()
                        .to_string();

                    if save_tool_note(&relative_path, current_tool_name, &content).is_ok() {
                        // 迁移成功，删除旧文件
                        let _ = fs::remove_file(&note_file);
                        println!(
                            "已迁移笔记: {} -> {}/{}",
                            note_file.display(),
                            tool_dir,
                            expected_file_name
                        );
                        return Some(content);
                    }
                }
            }
        }
    }

    None
}

/// 重命名工具笔记文件
pub fn rename_tool_note(tool_path: &str, old_name: &str, new_name: &str) -> Result<(), String> {
    if tool_path.is_empty() || old_name.is_empty() || new_name.is_empty() {
        return Ok(()); // 参数为空，无需处理
    }

    // 构建笔记文件路径
    let tool_dir = Path::new(&paths::get_resource_path()).join(tool_path);
    let old_note_file = tool_dir.join(format!("{}.md", old_name));
    let new_note_file = tool_dir.join(format!("{}.md", new_name));

    // 检查旧笔记文件是否存在
    if !old_note_file.exists() {
        return Ok(()); // 旧笔记不存在，无需重命名
    }

    // 检查新笔记文件是否已存在
    if new_note_file.exists() {
        // 新笔记文件已存在，创建一个备份
        let timestamp = chrono::Utc::now().timestamp();
        let backup_file = tool_dir.join(format!("{}_backup_{}.md", new_name, timestamp));
        fs::rename(&new_note_file, &backup_file).map_err(|e| format!("备份现有笔记失败: {}", e))?;
        println!("现有笔记已备份为: {}", backup_file.display());
    }

    // 重命名笔记文件
    fs::rename(&old_note_file, &new_note_file).map_err(|e| format!("重命名笔记文件失败: {}", e))?;

    println!(
        "已重命名笔记: {} -> {}",
        old_note_file.display(),
        new_note_file.display()
    );
    Ok(())
}

/// 清理工具对应的笔记文件，返回是否成功清理
pub fn clean_tool_note(tool_path: &str) -> bool {
    if tool_path.is_empty() {
        return false;
    }

    let path_parts: Vec<&str> = tool_path.split('/').collect();
    if path_parts.is_empty() {
        return false;
    }

    // 使用工具目录名作为ID
    let tool_dir_name = path_parts[path_parts.len() - 1];
    let mut tool_id = tool_dir_name.replace(' ', "_");
    tool_id = tool_id.replace('-', "_");

    // 尝试删除对应的笔记文件
    let notes_dir = paths::get_notes_dir();
    let note_file = notes_dir.join(format!("{}.md", tool_id));

    if note_file.exists() {
        if fs::remove_file(&note_file).is_ok() {
            println!("已清理无效工具的笔记文件: {}", note_file.display());
            return true;
        }
    }
    false
}

/// 迁移工具笔记（实际上是保持不变，因为新旧工具使用相同的目录名ID）
pub fn migrate_tool_note(tool_dir_name: &str) -> bool {
    let mut tool_id = tool_dir_name.replace(' ', "_");
    tool_id = tool_id.replace('-', "_");

    let notes_dir = paths::get_notes_dir();
    let note_file = notes_dir.join(format!("{}.md", tool_id));

    note_file.exists()
}
