use std::path::Path;
use std::process::Command;

/// 在工具目录打开终端（Windows 实现）
///
/// 对齐 macOS 版契约：cd 到工具目录；有命令执行命令，无命令列目录。
/// 用 `cmd /K` 启动一个新 cmd 窗口并保持打开。
pub fn open_terminal(dir: &Path, initial_command: Option<&str>) -> Result<(), String> {
    // 去掉路径中的双引号，防止 cmd 拼接时引号错配
    let dir_str = dir.to_string_lossy().replace('"', "");

    let cmd_line = match initial_command {
        Some(c) if !c.is_empty() => format!("cd /d \"{}\" && {}", dir_str, c),
        _ => format!("cd /d \"{}\" && dir", dir_str),
    };

    // start cmd /K "..." : 新开 cmd 窗口，执行命令后保持窗口
    Command::new("cmd")
        .args(["/C", "start", "cmd", "/K", &cmd_line])
        .spawn()
        .map_err(|e| format!("打开终端失败: {}", e))?;
    Ok(())
}
