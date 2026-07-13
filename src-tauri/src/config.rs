use crate::models::*;
use crate::paths::*;
use std::fs;
use std::path::Path;

/// 读取 YAML 配置文件，返回 (ConfigYaml, Categories)
/// 先解析为 YAML 结构体，再将 Categories 部分转换为 JSON 结构体
pub fn load_config() -> Result<(ConfigYaml, Categories), String> {
    let config_path = get_config_path();

    if !config_path.exists() {
        return Ok((ConfigYaml::default(), Categories::default()));
    }

    let data = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    // 先解析为 YAML 专用结构体（精确匹配 tool.yml 键名）
    let config_yaml: ConfigYaml = serde_yaml::from_str(&data)
        .map_err(|e| format!("解析Config失败: {}", e))?;

    // 转换 Categories 为 JSON 结构体
    let categories = Categories {
        categories: config_yaml
            .categories
            .iter()
            .map(|c| Category::from(c.clone()))
            .collect(),
    };

    Ok((config_yaml, categories))
}

/// 读取并返回 Categories（JSON IPC 用）
/// 先解析为 YAML 结构体，再转换为 JSON 结构体
pub fn get_categories() -> Result<Categories, String> {
    let config_path = get_config_path();

    if !config_path.exists() {
        return Ok(Categories::default());
    }

    let data = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    // 先解析为 YAML 专用结构体（精确匹配 tool.yml 键名）
    let config_yaml: ConfigYaml = serde_yaml::from_str(&data)
        .map_err(|e| format!("解析配置文件失败: {}", e))?;

    // 转换为 JSON 结构体（camelCase）
    let categories = Categories {
        categories: config_yaml
            .categories
            .into_iter()
            .map(Category::from)
            .collect(),
    };

    Ok(categories)
}

/// 保存分类配置到文件（原子写入，对齐 Go saveCategoriesToFile）
pub fn save_categories_to_file(categories: &Categories, config: &ConfigYaml) -> Result<(), String> {
    let config_path = get_config_path();

    // 确保配置文件所在目录存在
    let config_dir = config_path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(config_dir).map_err(|e| format!("创建配置目录失败: {}", e))?;

    // 创建备份
    let backup_path = format!("{}.backup", config_path.to_string_lossy());
    if config_path.exists() {
        if let Err(e) = fs::rename(&config_path, &backup_path) {
            eprintln!("创建备份失败: {}", e);
        } else {
            println!("已创建配置备份: {}", backup_path);
        }
    }

    // 检查Java配置是否为空，如果为空则尝试保持原有配置
    let mut java_config = config.java_paths.clone();
    if java_config.java8.is_empty() && java_config.java11.is_empty() && java_config.java17.is_empty() {
        // 尝试从备份中读取原有配置
        if Path::new(&backup_path).exists() {
            if let Ok(backup_data) = fs::read_to_string(&backup_path) {
                if let Ok(backup_config) = serde_yaml::from_str::<ConfigYaml>(&backup_data) {
                    if !backup_config.java_paths.java8.is_empty()
                        || !backup_config.java_paths.java11.is_empty()
                        || !backup_config.java_paths.java17.is_empty()
                    {
                        java_config = backup_config.java_paths;
                        println!("从备份中恢复Java配置");
                    }
                }
            }
        }

        // 如果备份也没有，使用默认的Java配置
        if java_config.java8.is_empty() && java_config.java11.is_empty() && java_config.java17.is_empty() {
            java_config = JavaConfig {
                java8: "resources/java8/bin/java".to_string(),
                java11: "resources/java11/bin/java".to_string(),
                java17: "resources/java17/bin/java".to_string(),
            };
            println!("使用默认Java配置");
        }
    }

    // 构建 YAML 结构
    let categories_yaml: Vec<CategoryYaml> = categories
        .categories
        .iter()
        .map(|c| CategoryYaml::from(c.clone()))
        .collect();

    let full_config = ConfigYaml {
        java_paths: java_config,
        categories: categories_yaml,
    };

    let yaml_data = serde_yaml::to_string(&full_config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    // 添加注释头部
    let content = format!(
        "# Java配置\n# 自定义Java路径配置，如果留空将使用系统默认Java\n{}",
        yaml_data
    );

    // 使用原子写入：先写入临时文件，然后重命名
    let temp_path = format!("{}.tmp", config_path.to_string_lossy());

    fs::write(&temp_path, &content).map_err(|e| format!("写入临时配置文件失败: {}", e))?;

    fs::rename(&temp_path, &config_path).map_err(|e| {
        // 如果重命名失败，清理临时文件
        let _ = fs::remove_file(&temp_path);
        format!("替换配置文件失败: {}", e)
    })?;

    println!("配置文件已更新: {}", config_path.to_string_lossy());

    // 验证写入的文件是否正确
    if let Err(e) = validate_config_file(&config_path) {
        eprintln!("警告：配置文件验证失败: {}", e);
        // 尝试恢复备份
        if Path::new(&backup_path).exists() {
            if fs::rename(&backup_path, &config_path).is_ok() {
                println!("已从备份恢复配置文件");
            }
        }
        return Err(format!("配置文件验证失败，请检查: {}", e));
    }

    Ok(())
}

/// 验证配置文件的完整性
pub fn validate_config_file(config_path: &Path) -> Result<(), String> {
    let data = fs::read_to_string(config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    // 尝试解析YAML
    let _config: ConfigYaml = serde_yaml::from_str(&data)
        .map_err(|e| format!("YAML解析失败: {}", e))?;

    // 检查是否有重复的Categories键
    let categories_count = data.matches("Categories:").count();
    if categories_count > 1 {
        return Err(format!("发现重复的Categories键，数量: {}", categories_count));
    }

    // 检查JavaPaths是否存在
    let java_paths_count = data.matches("javapath:").count();
    if java_paths_count != 1 {
        return Err(format!("JavaPaths键异常，数量: {}", java_paths_count));
    }

    Ok(())
}

/// 确保配置文件存在，不存在则创建或从旧位置迁移
pub fn ensure_config_exists() -> Result<(), String> {
    let new_path = get_config_path();

    // 用户配置已存在，无需处理
    if new_path.exists() {
        return Ok(());
    }

    // 确保配置目录存在
    let config_dir = get_config_dir();
    fs::create_dir_all(&config_dir).map_err(|e| format!("创建配置目录失败: {}", e))?;

    // 尝试从旧位置（.app 包内 Resources/tool.yml）迁移
    let old_path = Path::new(&get_resource_path()).join("tool.yml");
    if old_path.exists() {
        if let Ok(data) = fs::read(&old_path) {
            fs::write(&new_path, &data).map_err(|e| format!("迁移配置文件失败: {}", e))?;
            // 迁移旧笔记目录
            migrate_old_notes_dir();
            println!(
                "已从旧位置迁移配置文件: {} -> {}",
                old_path.display(),
                new_path.display()
            );
            return Ok(());
        }
    }

    // 首次安装：创建带默认 JavaPaths 的配置
    let default_config = ConfigYaml {
        java_paths: JavaConfig {
            java8: "resources/java8/bin/java".to_string(),
            java11: "resources/java11/bin/java".to_string(),
            java17: "resources/java17/bin/java".to_string(),
        },
        categories: vec![],
    };

    let categories = Categories {
        categories: vec![],
    };

    save_categories_to_file(&categories, &default_config)
        .map_err(|e| format!("创建默认配置失败: {}", e))?;

    println!("已创建默认配置文件: {}", new_path.display());
    Ok(())
}

/// 将旧的 notes 目录从应用包内迁移到配置目录
pub fn migrate_old_notes_dir() {
    let old_notes_dir = Path::new(&get_resource_path()).join("notes");
    let new_notes_dir = get_notes_dir();

    let files = match fs::read_dir(&old_notes_dir) {
        Ok(f) => f,
        Err(_) => return,
    };

    if fs::create_dir_all(&new_notes_dir).is_err() {
        return;
    }

    let mut migrated = 0;
    for file in files.flatten() {
        if file.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let old_file = file.path();
        let new_file = new_notes_dir.join(file.file_name());
        if let Ok(data) = fs::read(&old_file) {
            if fs::write(&new_file, &data).is_ok() {
                migrated += 1;
            }
        }
    }

    if migrated > 0 {
        println!("已迁移 {} 个笔记文件到配置目录", migrated);
    }
}

/// 获取工具的绝对路径
pub fn get_tool_absolute_path(tool_path: &str, file_name: &str) -> Result<String, String> {
    if tool_path.is_empty() {
        return Err("工具路径不能为空".to_string());
    }

    // 对于URL类型的路径，直接返回
    if tool_path.starts_with("http://") || tool_path.starts_with("https://") {
        return Ok(tool_path.to_string());
    }

    let full_path = if Path::new(tool_path).is_absolute() {
        PathBuf::from(tool_path)
    } else {
        PathBuf::from(get_resource_path()).join(tool_path)
    };

    // 如果有文件名，添加文件名
    let full_path = if !file_name.is_empty() {
        full_path.join(file_name)
    } else {
        full_path
    };

    // 返回清理后的绝对路径
    let abs_path = full_path
        .canonicalize()
        .or_else(|_| {
            // canonicalize 要求路径必须存在，如果不存在则手动构造
            if full_path.is_absolute() {
                Ok(full_path)
            } else {
                std::env::current_dir().map(|d| d.join(full_path))
            }
        })
        .map_err(|e| format!("获取绝对路径失败: {}", e))?;

    Ok(abs_path.to_string_lossy().to_string())
}

/// 清理工具路径（对齐 Go cleanToolPath）
pub fn clean_tool_path(path: &str) -> String {
    let original_path = path;

    // 如果是URL，直接返回原样
    if path.starts_with("http://") || path.starts_with("https://") {
        return path.to_string();
    }

    // 如果是绝对路径，直接返回原样
    if Path::new(path).is_absolute() {
        return path.to_string();
    }

    let mut p = path.to_string();

    // 1. 处理包含 "/Contents/Resources/" 的错误拼接路径
    if p.contains("/Contents/Resources/") {
        if let Some(last_index) = p.rfind("/Contents/Resources/") {
            let suffix = &p[last_index + "/Contents/Resources/".len()..];
            if suffix.starts_with("resources/") {
                p = suffix.to_string();
            } else {
                p = format!("resources/{}", suffix);
            }
        }
    }

    // 2. 移除开头的多余斜杠
    while p.starts_with('/') {
        p = p[1..].to_string();
    }

    // 3. 确保相对路径以 resources/ 开头
    if !p.starts_with("resources/") {
        p = format!("resources/{}", p);
    }

    // 4. 移除重复的resources前缀
    while p.contains("resources/resources/") {
        p = p.replace("resources/resources/", "resources/");
    }

    // 5. 清理路径中的重复斜杠 — 使用路径规范化
    // Go 的 filepath.Clean 会移除多余的斜杠和 .. / .
    p = clean_path_str(&p);

    if original_path != p {
        println!("路径清理: {} -> {}", original_path, p);
    }

    p
}

/// 模拟 Go filepath.Clean 的行为
fn clean_path_str(path: &str) -> String {
    use std::path::PathBuf;
    // 将 / 分隔的路径转换为 PathBuf 并 normalize
    // 注意：在 Windows 上分隔符不同，但这里处理的是配置文件中的相对路径，始终用 /
    let pb = PathBuf::from(path);
    pb.to_string_lossy().to_string()
}

use std::path::PathBuf;
