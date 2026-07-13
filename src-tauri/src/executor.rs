use crate::config;
use crate::platform;
use std::path::Path;
use std::process::Command;

/// 执行工具命令（支持自定义命令）
/// 对齐 Go app.go:217 ExecuteCommandWithCustom
pub fn execute_command_with_custom(
    path: &str,
    optional: &str,
    value: &str,
    filename: &str,
    custom_command: &str,
    _java_path: &str,
) -> Result<(), String> {
    // 对于浏览器打开类型，保持URL原样，其他类型再进行路径清理
    let mut path = path.to_string();
    if value.to_lowercase() != "browser" {
        let cleaned_path = config::clean_tool_path(&path);
        if cleaned_path != path {
            println!("执行时路径已清理: {} -> {}", path, cleaned_path);
            path = cleaned_path;
        }
    }

    // 读取配置文件
    let (config_yaml, _categories) = config::load_config()?;

    // 获取工具的绝对路径
    let tool_path = config::get_tool_absolute_path(&path, "")?;
    println!("工具绝对路径: {}", tool_path);

    match value {
        "Java8" | "Java11" | "Java17" => {
            // 构建Java可执行文件路径
            let java_exec = match value {
                "Java8" if !config_yaml.java_paths.java8.is_empty() => &config_yaml.java_paths.java8,
                "Java11" if !config_yaml.java_paths.java11.is_empty() => &config_yaml.java_paths.java11,
                "Java17" if !config_yaml.java_paths.java17.is_empty() => &config_yaml.java_paths.java17,
                _ => "java",
            };

            let jar_path = Path::new(&tool_path).join(filename);

            println!("Java可执行文件: {}", java_exec);
            println!("工具目录: {}", tool_path);
            println!("JAR文件: {}", jar_path.display());
            println!("可选参数: {}", optional);

            // 检查Java可执行文件是否存在（仅当不是系统java时）
            if java_exec != "java" {
                if !Path::new(java_exec).exists() {
                    return Err(format!("java可执行文件不存在: {}", java_exec));
                }
            }

            // 检查JAR文件是否存在
            if !jar_path.exists() {
                return Err(format!("JAR文件不存在: {}", jar_path.display()));
            }

            // 构建参数列表
            let mut args: Vec<String> = vec!["-jar".to_string(), filename.to_string()];
            let trimmed_optional = optional.trim();
            if !trimmed_optional.is_empty() {
                let mut optional_args: Vec<String> = trimmed_optional
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
                optional_args.append(&mut args);
                args = optional_args;
            }

            let mut java_cmd = Command::new(java_exec);
            java_cmd.args(&args);
            java_cmd.current_dir(&tool_path);
            platform::spawn_hidden(java_cmd).map_err(|e| format!("启动Java工具失败: {}", e))?;
            println!("Java工具已后台启动");
            Ok(())
        }

        "Open" => {
            let mut open_cmd = Command::new("open");
            open_cmd.arg(filename);
            open_cmd.current_dir(&tool_path);
            platform::spawn_hidden(open_cmd).map_err(|e| format!("打开文件失败: {}", e))?;
            Ok(())
        }

        "openterm" => {
            // 检查是否有自定义命令
            if !custom_command.is_empty() {
                // 有自定义命令，替换占位符
                let mut command = custom_command.to_string();
                if !filename.is_empty() {
                    let file_path = Path::new(&tool_path).join(filename);
                    command = command.replace("{file}", &file_path.to_string_lossy());
                    command = command.replace("{filename}", filename);
                }
                command = command.replace("{path}", &tool_path);

                println!("终端自定义命令: {}", command);
                println!("工具目录: {}", tool_path);

                // 在终端中执行自定义命令
                return platform::open_terminal(Path::new(&tool_path), Some(&command));
            }
            // 没有自定义命令，默认打开终端
            platform::open_terminal(Path::new(&tool_path), None)
        }

        "Browser" => {
            // 直接使用系统默认浏览器打开URL或文件
            let target;
            // 如果是URL，直接打开
            if path.starts_with("http://") || path.starts_with("https://") {
                target = path.clone();
            } else {
                // 非URL：使用工具绝对路径（如果有文件名优先打开文件）
                if !filename.is_empty() {
                    target = Path::new(&tool_path)
                        .join(filename)
                        .to_string_lossy()
                        .to_string();
                } else {
                    target = tool_path.clone();
                }
            }
            platform::open_url(&target)
        }

        "Binary" => {
            let binary_path = Path::new(&tool_path).join(filename);
            if !binary_path.exists() {
                return Err(format!("二进制文件不存在: {}", binary_path.display()));
            }
            let mut bin_cmd = Command::new(&binary_path);
            bin_cmd.current_dir(&tool_path);
            platform::spawn_hidden(bin_cmd).map_err(|e| format!("启动二进制文件失败: {}", e))?;
            println!("二进制文件已后台启动");
            Ok(())
        }

        _ => Err(format!("不支持的命令类型: {}", value)),
    }
}
