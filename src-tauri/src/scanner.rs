use crate::config;
use crate::models::*;
use crate::paths::get_resource_path;
use std::fs;
use std::path::Path;

/// 读取现有的分类配置，返回目录名到分类信息的映射
pub fn load_existing_categories() -> std::collections::HashMap<String, CategoryInfo> {
    let mut existing_categories = std::collections::HashMap::new();

    if let Ok(categories) = config::get_categories() {
        for category in &categories.categories {
            let possible_dir_names = get_possible_dir_names(&category.name);
            let category_info = CategoryInfo {
                name: category.name.clone(),
                icon: category.icon.clone().unwrap_or_default(),
            };
            for dir_name in possible_dir_names {
                existing_categories.insert(dir_name, category_info.clone());
            }
        }
    }

    existing_categories
}

/// 根据分类名获取可能的目录名
pub fn get_possible_dir_names(category_name: &str) -> Vec<String> {
    let mapping: &[(&str, &[&str])] = &[
        ("信息收集", &["info", "information", "recon"]),
        ("渗透利器", &["pentest", "penetration", "exploit"]),
        ("Webshell管理工具", &["webshell", "shell", "backdoor"]),
        ("框架利用工具", &["framework", "comprehensive", "exploit"]),
        ("数据库利用", &["databases", "database", "db"]),
        ("代理", &["proxy", "proxies"]),
        ("代理工具", &["proxy", "proxies"]),
        ("内网工具", &["intranet", "Intranet"]),
        ("其他", &["other", "misc", "miscellaneous"]),
    ];

    for (name, dir_names) in mapping {
        if category_name == *name {
            return dir_names.iter().map(|s| s.to_string()).collect();
        }
    }

    // 如果没有映射，尝试简单的转换
    let simplified = category_name.replace(' ', "").to_lowercase();
    vec![simplified, category_name.to_string()]
}

/// 获取分类信息，优先使用现有配置
pub fn get_category_info(
    dir_name: &str,
    existing_categories: &std::collections::HashMap<String, CategoryInfo>,
) -> CategoryInfo {
    if let Some(info) = existing_categories.get(dir_name) {
        return info.clone();
    }
    CategoryInfo {
        name: dir_name.to_string(),
        icon: String::new(),
    }
}

/// 扫描指定路径下的工具
/// resources文件夹下的每个目录是分类文件夹，每个分类下的子目录是工具文件夹
/// 只收录包含可启动文件（jar、二进制、exe）的工具目录
pub fn scan_tools_in_path(scan_path: &str) -> Result<Vec<ScannedTool>, String> {
    let mut scanned_tools = Vec::new();

    // 检查扫描目录是否存在
    if !Path::new(scan_path).exists() {
        return Err(format!("扫描目录不存在: {}", scan_path));
    }

    // 读取现有的tool.yml配置
    let existing_categories = load_existing_categories();

    // 遍历resources文件夹下的分类文件夹
    let category_dirs = fs::read_dir(scan_path)
        .map_err(|e| format!("读取扫描目录失败: {}", e))?;

    for category_dir in category_dirs.flatten() {
        let file_type = match category_dir.file_type() {
            Ok(t) if t.is_dir() => true,
            _ => continue,
        };
        if !file_type {
            continue;
        }

        let dir_name = category_dir.file_name().to_string_lossy().to_string();

        // 跳过特殊目录（Java环境目录）
        if dir_name == "java8" || dir_name == "java11" || dir_name == "java17" {
            continue;
        }

        let category_path = category_dir.path();
        let category_info = get_category_info(&dir_name, &existing_categories);

        // 遍历分类目录下的工具文件夹
        let tool_dirs = match fs::read_dir(&category_path) {
            Ok(d) => d,
            Err(_) => continue, // 跳过无法读取的目录
        };

        for tool_dir in tool_dirs.flatten() {
            let is_dir = tool_dir
                .file_type()
                .map(|t| t.is_dir())
                .unwrap_or(false);
            if !is_dir {
                continue;
            }

            // 只保留包含 jar、二进制或 exe 的工具目录
            let tool_dir_path = tool_dir.path();
            if detect_launchable_file(&tool_dir_path).is_none() {
                continue;
            }

            // 构建相对于resources的路径 - 确保始终保存相对路径格式
            let tool_path = format!("resources/{}/{}", dir_name, tool_dir.file_name().to_string_lossy());
            // 确保路径分隔符统一（已经是 / 分隔）

            let scanned_tool = ScannedTool {
                path: tool_path,
                category: category_info.name.clone(),
                possible_files: vec![],
            };
            scanned_tools.push(scanned_tool);
        }
    }

    Ok(scanned_tools)
}

/// 扫描自定义路径下的工具（使用绝对路径）
/// 支持两种目录结构：
/// 1. 分类式：customPath/category1/tool1, customPath/category2/tool2
/// 2. 平铺式：customPath/tool1, customPath/tool2 (统一归类为"自定义工具")
/// 只收录包含可启动文件（jar、二进制、exe）的工具目录
pub fn scan_tools_in_custom_path(scan_path: &str) -> Result<Vec<ScannedTool>, String> {
    let mut scanned_tools = Vec::new();

    // 检查扫描目录是否存在
    if !Path::new(scan_path).exists() {
        return Err(format!("扫描目录不存在: {}", scan_path));
    }

    // 读取现有的tool.yml配置
    let existing_categories = load_existing_categories();

    // 先尝试分类式扫描
    let mut category_scanned = false;
    let entries = fs::read_dir(scan_path)
        .map_err(|e| format!("读取扫描目录失败: {}", e))?;

    let entry_vec: Vec<_> = entries.flatten().collect();

    // 检查是否是分类式结构（存在目录，且目录下还有子目录）
    for entry in &entry_vec {
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if !is_dir {
            continue;
        }

        let category_path = entry.path();
        let sub_entries = match fs::read_dir(&category_path) {
            Ok(d) => d,
            Err(_) => continue,
        };

        let sub_vec: Vec<_> = sub_entries.flatten().collect();

        // 检查是否有子目录（工具目录）
        let has_sub_dirs = sub_vec.iter().any(|e| {
            e.file_type().map(|t| t.is_dir()).unwrap_or(false)
        });

        if has_sub_dirs {
            // 是分类式结构，按分类扫描
            let category_info = get_category_info(
                &entry.file_name().to_string_lossy(),
                &existing_categories,
            );

            for sub_entry in &sub_vec {
                let is_dir = sub_entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                if !is_dir {
                    continue;
                }
                let tool_abs_path = category_path.join(sub_entry.file_name());

                // 只保留包含 jar、二进制或 exe 的工具目录
                if detect_launchable_file(&tool_abs_path).is_none() {
                    continue;
                }

                let scanned_tool = ScannedTool {
                    path: tool_abs_path.to_string_lossy().to_string(),
                    category: category_info.name.clone(),
                    possible_files: vec![],
                };
                scanned_tools.push(scanned_tool);
            }
            category_scanned = true;
        }
    }

    // 如果不是分类式结构，进行平铺式扫描
    if !category_scanned {
        let default_category = "自定义工具";
        let category_info = get_category_info(default_category, &existing_categories);

        for entry in &entry_vec {
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            if !is_dir {
                continue;
            }
            let tool_abs_path = scan_path.to_string() + "/" + &entry.file_name().to_string_lossy();

            // 只保留包含 jar、二进制或 exe 的工具目录
            if detect_launchable_file(Path::new(&tool_abs_path)).is_none() {
                continue;
            }

            let scanned_tool = ScannedTool {
                path: tool_abs_path,
                category: category_info.name.clone(),
                possible_files: vec![],
            };
            scanned_tools.push(scanned_tool);
        }
    }

    Ok(scanned_tools)
}

/// 检测工具目录中的可启动文件（jar、exe、二进制），按 jar > exe/二进制 的优先级返回
/// 返回 (toolType, fileName, command)，目录中没有可启动文件时返回 None
pub fn detect_launchable_file(tool_dir: &Path) -> Option<(String, String, String)> {
    let files = fs::read_dir(tool_dir).ok()?;

    let mut jar_file: Option<String> = None;
    let mut bin_file: Option<String> = None;

    for file in files.flatten() {
        if file.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let file_name = file.file_name().to_string_lossy().to_string();
        let file_name_lower = file_name.to_lowercase();

        if file_name_lower.ends_with(".jar") {
            if jar_file.is_none() {
                jar_file = Some(file_name);
            }
        } else if file_name_lower.ends_with(".exe") || is_binary_executable(&file_name) {
            // Unix 上无扩展名二进制需要执行权限；.exe 在 Windows 上没有执行位概念
            #[cfg(unix)]
            let launchable = if file_name_lower.ends_with(".exe") {
                true
            } else {
                file.metadata()
                    .map(|meta| {
                        use std::os::unix::fs::PermissionsExt;
                        meta.permissions().mode() & 0o111 != 0
                    })
                    .unwrap_or(false)
            };
            #[cfg(not(unix))]
            let launchable = true;

            if launchable && bin_file.is_none() {
                bin_file = Some(file_name);
            }
        }
    }

    jar_file
        .map(|f| ("Java8".to_string(), f, "-jar".to_string()))
        .or_else(|| bin_file.map(|f| ("Binary".to_string(), f, String::new())))
}

/// 分析工具目录内容，决定如何添加工具
/// 返回 (toolType, fileName, command)
pub fn analyze_tool_directory(tool_dir: &str) -> (String, String, String) {
    let dir_path = Path::new(tool_dir);

    // 目录为空或无法读取时返回openterm
    let has_entries = fs::read_dir(dir_path)
        .map(|mut entries| entries.next().is_some())
        .unwrap_or(false);
    if !has_entries {
        return ("openterm".to_string(), "".to_string(), "".to_string());
    }

    // 优先级：jar > 二进制/exe
    if let Some(result) = detect_launchable_file(dir_path) {
        return result;
    }

    // 查找.app目录（macOS应用程序包）
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            if !is_dir {
                continue;
            }
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.to_lowercase().ends_with(".app") {
                return ("Open".to_string(), file_name, "".to_string());
            }
        }
    }

    // 如果只有子目录或其他文件，使用openterm
    ("openterm".to_string(), "".to_string(), "".to_string())
}

/// 判断文件是否为可执行文件
// is_dir/_metadata 仅在 Unix 分支使用，Windows 上编译掉会触发未使用警告
#[cfg_attr(not(unix), allow(unused_variables))]
pub fn is_executable_file(file_name: &str, is_dir: bool, _metadata: Option<&fs::Metadata>) -> bool {
    let file_name_lower = file_name.to_lowercase();

    // 检查常见的可执行文件扩展名
    if file_name_lower.ends_with(".jar")
        || file_name_lower.ends_with(".exe")
        || file_name_lower.ends_with(".app")
        || file_name_lower.ends_with(".sh")
        || file_name_lower.ends_with(".py")
        || file_name_lower.ends_with(".bat")
        || file_name_lower.ends_with(".cmd")
    {
        return true;
    }

    // 在Unix系统上，检查文件是否有执行权限（无扩展名的二进制文件）
    #[cfg(unix)]
    {
        if !is_dir {
            if let Some(meta) = _metadata {
                use std::os::unix::fs::PermissionsExt;
                let mode = meta.permissions().mode();
                if mode & 0o111 != 0 {
                    // 进一步检查是否为二进制文件（排除脚本文件）
                    return is_binary_executable(file_name);
                }
            }
        }
    }

    false
}

/// 判断是否为二进制可执行文件
pub fn is_binary_executable(file_name: &str) -> bool {
    // 简单检查：无扩展名且不是常见的文本文件
    let path = Path::new(file_name);
    let ext = path.extension();

    if ext.is_none() {
        // 排除常见的配置文件和文档
        let lower_name = file_name.to_lowercase();
        let exclude_patterns = [
            "readme",
            "license",
            "changelog",
            "makefile",
            "dockerfile",
            ".gitignore",
            ".gitattributes",
        ];
        for pattern in &exclude_patterns {
            if lower_name.contains(pattern) {
                return false;
            }
        }
        return true;
    }
    false
}

/// 格式化工具名称
pub fn format_tool_name(dir_name: &str) -> String {
    let mut name = dir_name.replace('_', " ");
    name = name.replace('-', " ");

    // 首字母大写
    if !name.is_empty() {
        let mut chars = name.chars();
        let first = chars.next().unwrap();
        name = first.to_uppercase().collect::<String>() + chars.as_str();
    }

    name
}

/// 获取真正的新工具（过滤掉已存在的）
pub fn get_new_tools_from_scanned(tools: &[ScannedTool]) -> Result<Vec<ScannedTool>, String> {
    let (_config_yaml, categories) = config::load_config().unwrap_or_default();

    // 获取现有工具的路径作为唯一标识
    let mut existing_tool_paths = std::collections::HashSet::new();
    let mut existing_tool_path_file_names = std::collections::HashSet::new();

    for category in &categories.categories {
        for tool in &category.tools {
            existing_tool_paths.insert(tool.path.clone());
            if !tool.file_name.is_empty() {
                let combined_path = format!("{}/{}", tool.path, tool.file_name);
                existing_tool_path_file_names.insert(combined_path);
            }
        }
    }

    // 过滤出真正的新工具
    let mut new_tools = Vec::new();
    for scanned_tool in tools {
        let scanned_path = scanned_tool.path.replace('\\', "/");
        let mut is_existing = existing_tool_paths.contains(&scanned_path);

        // 如果路径不存在，还需要检查是否有Path+FileName的组合匹配
        if !is_existing {
            let path_parts: Vec<&str> = scanned_path.split('/').collect();
            if !path_parts.is_empty() {
                let last_part = path_parts[path_parts.len() - 1];
                if last_part.to_lowercase().ends_with(".app") {
                    // 如果是.app，检查父目录+文件名是否已存在
                    if path_parts.len() > 1 {
                        let parent_path = path_parts[..path_parts.len() - 1].join("/");
                        let combined_path = format!("{}/{}", parent_path, last_part);
                        is_existing = existing_tool_path_file_names.contains(&combined_path);
                    }
                } else {
                    // 检查是否有工具使用这个路径作为Path，FileName为目录名
                    for existing_path in &existing_tool_paths {
                        if existing_path.starts_with(&format!("{}/", scanned_path)) {
                            let rel_path = existing_path[scanned_path.len() + 1..].to_string();
                            if !rel_path.is_empty() && !rel_path.contains('/') {
                                is_existing = true;
                                break;
                            }
                        }
                    }
                }
            }
        }

        if !is_existing {
            new_tools.push(scanned_tool.clone());
        }
    }

    Ok(new_tools)
}

/// 自动添加扫描到的工具
pub fn auto_add_scanned_tools(tools: &[ScannedTool]) -> Result<(), String> {
    let (config_yaml, mut categories) = config::load_config().unwrap_or_default();

    // 建立现有分类的映射，保留图标信息
    let mut existing_category_map: std::collections::HashMap<String, Category> =
        std::collections::HashMap::new();
    for category in &categories.categories {
        existing_category_map.insert(category.name.clone(), category.clone());
    }

    // 获取现有工具的路径作为唯一标识
    let mut existing_tool_paths = std::collections::HashSet::new();
    let mut existing_tool_path_file_names = std::collections::HashSet::new();

    for category in &categories.categories {
        for tool in &category.tools {
            existing_tool_paths.insert(tool.path.clone());
            if !tool.file_name.is_empty() {
                let combined_path = format!("{}/{}", tool.path, tool.file_name);
                existing_tool_path_file_names.insert(combined_path);
            }
        }
    }

    // 添加新发现的工具
    let base_path = get_resource_path();
    for scanned_tool in tools {
        let scanned_path = scanned_tool.path.replace('\\', "/");
        let mut is_existing = existing_tool_paths.contains(&scanned_path);

        // 如果路径不存在，还需要检查是否有Path+FileName的组合匹配
        if !is_existing {
            let path_parts: Vec<&str> = scanned_path.split('/').collect();
            if !path_parts.is_empty() {
                let last_part = path_parts[path_parts.len() - 1];
                if last_part.to_lowercase().ends_with(".app") {
                    if path_parts.len() > 1 {
                        let parent_path = path_parts[..path_parts.len() - 1].join("/");
                        let combined_path = format!("{}/{}", parent_path, last_part);
                        is_existing = existing_tool_path_file_names.contains(&combined_path);
                    }
                } else {
                    for existing_path in &existing_tool_paths {
                        if existing_path.starts_with(&format!("{}/", scanned_path)) {
                            let rel_path = existing_path[scanned_path.len() + 1..].to_string();
                            if !rel_path.is_empty() && !rel_path.contains('/') {
                                is_existing = true;
                                break;
                            }
                        }
                    }
                }
            }
        }

        if is_existing {
            println!("跳过已存在的工具: 路径: {}", scanned_tool.path);
            continue;
        }

        // 从路径中提取工具名称（使用文件夹名）
        let path_parts: Vec<&str> = scanned_tool.path.split('/').collect();
        let tool_name = if path_parts.is_empty() || path_parts[path_parts.len() - 1].is_empty() {
            "Unknown Tool".to_string()
        } else {
            path_parts[path_parts.len() - 1].to_string()
        };

        // 分析工具目录内容，决定如何添加工具
        let full_tool_path = Path::new(&base_path).join(&scanned_tool.path);
        let (tool_type, file_name, command) =
            analyze_tool_directory(&full_tool_path.to_string_lossy());

        // 创建新工具
        let new_tool = Tool {
            name: format_tool_name(&tool_name),
            path: scanned_tool.path.clone(),
            file_name,
            value: tool_type,
            command,
            optional: String::new(),
            description: Some(format!("扫描发现的工具路径: {}", scanned_tool.path)),
            ..Default::default()
        };

        // 查找或创建分类
        let mut category_found = false;
        for category in &mut categories.categories {
            if category.name == scanned_tool.category {
                category.tools.push(new_tool.clone());
                category_found = true;
                break;
            }
        }

        if !category_found {
            // 创建新分类，如果有现有分类信息则保留图标
            let mut new_category = Category {
                name: scanned_tool.category.clone(),
                icon: None,
                tools: vec![new_tool],
            };
            if let Some(existing_category) = existing_category_map.get(&scanned_tool.category) {
                new_category.icon = existing_category.icon.clone();
            }
            categories.categories.push(new_category);
        }
    }

    // 保存配置
    config::save_categories_to_file(&categories, &config_yaml)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    /// 在系统临时目录下创建一个唯一的测试目录
    fn make_temp_dir(label: &str) -> std::path::PathBuf {
        let dir = env::temp_dir().join(format!(
            "spearx_scanner_test_{}_{}",
            label,
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_file(dir: &Path, name: &str) {
        fs::write(dir.join(name), b"test").unwrap();
    }

    #[cfg(unix)]
    fn set_exec_bit(path: &Path) {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path).unwrap().permissions();
        perms.set_mode(perms.mode() | 0o111);
        fs::set_permissions(path, perms).unwrap();
    }

    #[test]
    fn test_detect_jar_file() {
        let dir = make_temp_dir("jar");
        write_file(&dir, "tool.jar");
        write_file(&dir, "readme.txt");

        let result = detect_launchable_file(&dir).unwrap();
        assert_eq!(result.0, "Java8");
        assert_eq!(result.1, "tool.jar");
        assert_eq!(result.2, "-jar");

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_detect_exe_file() {
        let dir = make_temp_dir("exe");
        write_file(&dir, "tool.exe");
        write_file(&dir, "readme.txt");

        let result = detect_launchable_file(&dir).unwrap();
        assert_eq!(result.0, "Binary");
        assert_eq!(result.1, "tool.exe");

        fs::remove_dir_all(&dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn test_detect_unix_binary_requires_exec_bit() {
        let dir = make_temp_dir("bin");
        write_file(&dir, "toolbin");
        write_file(&dir, "readme.txt");

        // 无执行权限的裸文件不算可启动文件
        assert!(detect_launchable_file(&dir).is_none());

        set_exec_bit(&dir.join("toolbin"));
        let result = detect_launchable_file(&dir).unwrap();
        assert_eq!(result.0, "Binary");
        assert_eq!(result.1, "toolbin");

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_detect_none_for_docs_only_dir() {
        let dir = make_temp_dir("none");
        write_file(&dir, "readme.txt");
        write_file(&dir, "LICENSE");

        assert!(detect_launchable_file(&dir).is_none());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_analyze_priority_jar_over_exe() {
        let dir = make_temp_dir("prio");
        write_file(&dir, "b.exe");
        write_file(&dir, "a.jar");

        let result = analyze_tool_directory(&dir.to_string_lossy());
        assert_eq!(result.0, "Java8");
        assert_eq!(result.1, "a.jar");

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_analyze_app_bundle_falls_back_to_open() {
        let dir = make_temp_dir("app");
        fs::create_dir(dir.join("SomeApp.app")).unwrap();

        let result = analyze_tool_directory(&dir.to_string_lossy());
        assert_eq!(result.0, "Open");
        assert_eq!(result.1, "SomeApp.app");

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_analyze_empty_dir_returns_openterm() {
        let dir = make_temp_dir("empty");
        let result = analyze_tool_directory(&dir.to_string_lossy());
        assert_eq!(result.0, "openterm");

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_scan_tools_only_keeps_launchable_dirs() {
        let root = make_temp_dir("scan");
        let cat = root.join("info");
        let with_jar = cat.join("tool-with-jar");
        let without = cat.join("tool-without");
        fs::create_dir_all(&with_jar).unwrap();
        fs::create_dir_all(&without).unwrap();
        write_file(&with_jar, "tool.jar");
        write_file(&without, "readme.txt");

        let scanned = scan_tools_in_path(&root.to_string_lossy()).unwrap();
        assert_eq!(scanned.len(), 1);
        assert!(scanned[0].path.ends_with("tool-with-jar"));
        // 分类名可能被真实用户配置中的目录映射改写（info -> 信息收集 等），不在此断言

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn test_scan_custom_path_flat_keeps_launchable_dirs() {
        let root = make_temp_dir("custom_flat");
        let with_exe = root.join("tool-with-exe");
        let without = root.join("tool-without");
        fs::create_dir_all(&with_exe).unwrap();
        fs::create_dir_all(&without).unwrap();
        write_file(&with_exe, "tool.exe");
        write_file(&without, "readme.txt");

        let scanned = scan_tools_in_custom_path(&root.to_string_lossy()).unwrap();
        assert_eq!(scanned.len(), 1);
        assert!(scanned[0].path.ends_with("tool-with-exe"));
        assert_eq!(scanned[0].category, "自定义工具");

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn test_scan_custom_path_categorised_keeps_launchable_dirs() {
        let root = make_temp_dir("custom_cat");
        let cat = root.join("pentest");
        let with_bin = cat.join("tool-with-bin");
        let without = cat.join("tool-without");
        fs::create_dir_all(&with_bin).unwrap();
        fs::create_dir_all(&without).unwrap();

        #[cfg(unix)]
        {
            write_file(&with_bin, "toolbin");
            set_exec_bit(&with_bin.join("toolbin"));
        }
        #[cfg(not(unix))]
        write_file(&with_bin, "tool.exe");
        write_file(&without, "readme.txt");

        let scanned = scan_tools_in_custom_path(&root.to_string_lossy()).unwrap();
        assert_eq!(scanned.len(), 1);
        assert!(scanned[0].path.ends_with("tool-with-bin"));
        assert_eq!(scanned[0].category, "pentest");

        fs::remove_dir_all(&root).unwrap();
    }
}
