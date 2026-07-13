use std::path::Path;
use std::process::Command;

/// 打开终端并执行命令（macOS 专属）
///
/// 检测 iTerm.app 是否存在：
/// - 存在：通过 AppleScript 在 iTerm 中创建新窗口并执行命令
/// - 不存在：通过 AppleScript 在 Terminal.app 中执行命令
///
/// 有自定义命令时执行 `cd dir && cmd`，否则执行 `cd dir && ls --color=always`
pub fn open_terminal(dir: &Path, initial_command: Option<&str>) -> Result<(), String> {
    let dir_str = dir.display().to_string();
    // 对路径中的单引号做转义，防止 AppleScript 注入
    let dir_escaped = dir_str.replace('\'', "'\\''");

    let command_to_run = match initial_command {
        Some(cmd) if !cmd.is_empty() => {
            let cmd_escaped = cmd.replace('\'', "'\\''");
            format!("cd '{}' && {}", dir_escaped, cmd_escaped)
        }
        _ => format!("cd '{}' && ls --color=always", dir_escaped),
    };

    let iterm_exists = Path::new("/Applications/iTerm.app").exists();

    let script = if iterm_exists {
        format!(
            r#"tell application "iTerm"
                create window with default profile
                tell current session of current window
                    write text "{}"
                end tell
            end tell"#,
            command_to_run
        )
    } else {
        format!(
            r#"tell application "Terminal"
                do script "{}"
            end tell"#,
            command_to_run
        )
    };

    Command::new("osascript")
        .args(["-e", &script])
        .spawn()
        .map_err(|e| format!("打开终端失败: {}", e))?;
    Ok(())
}
