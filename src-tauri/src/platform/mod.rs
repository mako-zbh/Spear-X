#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
pub use macos::*;
#[cfg(target_os = "windows")]
pub use windows::*;
#[cfg(target_os = "linux")]
pub use linux::*;

use std::process::Command;

/// 隐藏控制台窗口启动进程（对齐 Go setHideWindow）
///
/// Windows 下设置 CREATE_NO_WINDOW (0x08000000) 标志，
/// 阻止子进程创建新的控制台窗口。
/// macOS/Linux 的 GUI 应用中 spawn 不会弹出终端窗口，无需特殊处理。
pub fn spawn_hidden(mut cmd: Command) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.spawn()?;
    Ok(())
}

/// 打开文件管理器（对齐 Go OpenToolDirectory）
///
/// macOS: open, Windows: explorer, Linux: xdg-open
pub fn open_in_file_manager(path: &str) -> Result<(), String> {
    let cmd = match std::env::consts::OS {
        "macos" => {
            let mut c = Command::new("open");
            c.arg(path);
            c
        }
        "windows" => {
            let mut c = Command::new("explorer");
            c.arg(path);
            c
        }
        _ => {
            let mut c = Command::new("xdg-open");
            c.arg(path);
            c
        }
    };
    spawn_hidden(cmd).map_err(|e| format!("打开目录失败: {}", e))
}

/// 用系统默认程序打开 URL（对齐 Go Browser case + OpenGitHubPage）
///
/// macOS: open, Windows: cmd /C start, Linux: xdg-open
pub fn open_url(url: &str) -> Result<(), String> {
    let cmd = match std::env::consts::OS {
        "macos" => {
            let mut c = Command::new("open");
            c.arg(url);
            c
        }
        "windows" => {
            let mut c = Command::new("cmd");
            c.args(["/C", "start", "", url]);
            c
        }
        _ => {
            let mut c = Command::new("xdg-open");
            c.arg(url);
            c
        }
    };
    spawn_hidden(cmd).map_err(|e| format!("打开浏览器失败: {}", e))
}
