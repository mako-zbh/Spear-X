#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use spearx_lib::commands;
use spearx_lib::config;
use spearx_lib::maintenance;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // macOS: 整窗原生磨砂玻璃。底色全部交给原生层，WebView 完全透明，
            // 这样 resize 时露出的层与稳态显示的层是同一个，从根本上消除白闪/色差。
            #[cfg(target_os = "macos")]
            {
                let window = app.get_webview_window("main").unwrap();

                // 1) 整窗毛玻璃（UnderWindowBackground 跟随窗口外观）
                let _ = window_vibrancy::apply_vibrancy(
                    &window,
                    window_vibrancy::NSVisualEffectMaterial::UnderWindowBackground,
                    Some(window_vibrancy::NSVisualEffectState::Active),
                    None,
                );

                // 2) 强制窗口外观为 Dark：UnderWindowBackground 在浅色模式下会变浅，
                //    要"深色磨砂玻璃"必须锁住 appearance，不随系统切换。
                use objc2_app_kit::{
                    NSAppearance, NSAppearanceCustomization, NSAppearanceNameVibrantDark,
                    NSColor, NSView,
                };
                use raw_window_handle::{HasWindowHandle, RawWindowHandle};
                match window.window_handle() {
                    Ok(handle) => {
                        if let RawWindowHandle::AppKit(appkit) = handle.as_raw() {
                            let ns_view_ptr = appkit.ns_view.as_ptr() as *mut objc2::runtime::AnyObject;
                            if !ns_view_ptr.is_null() {
                                unsafe {
                                    let ns_view: &NSView =
                                        &*(ns_view_ptr as *const NSView);
                                    if let Some(ns_window) = ns_view.window() {
                                        // 标记非不透明 + 清空窗口自带背景，
                                        // 让只有 vibrancy 那一层负责"底"。
                                        ns_window.setOpaque(false);
                                        ns_window.setBackgroundColor(Some(&NSColor::clearColor()));

                                        // 强制深色外观，保证磨砂为深色（不随系统切换）
                                        if let Some(dark) =
                                            NSAppearance::appearanceNamed(&NSAppearanceNameVibrantDark)
                                        {
                                            ns_window.setAppearance(Some(&dark));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => eprintln!("获取窗口句柄失败: {}", e),
                }
            }

            // 首次启动：确保配置文件存在
            if let Err(e) = config::ensure_config_exists() {
                eprintln!("初始化配置文件失败: {}", e);
            }

            // 启动时自动检测和修复配置
            tauri::async_runtime::spawn(async move {
                println!("正在进行启动时配置检查和修复...");

                // 1. 修复配置文件格式问题
                if let Err(e) = maintenance::repair_config_file() {
                    eprintln!("配置文件修复失败: {}", e);
                }

                // 2. 清理和修复工具路径
                if let Err(e) = maintenance::cleanup_tool_paths() {
                    eprintln!("路径修复失败: {}", e);
                }

                println!("启动时配置检查和修复完成");
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 配置
            commands::config_cmd::get_categories,
            commands::config_cmd::get_java_config,
            commands::config_cmd::save_java_config,
            // 执行
            commands::execution::execute_command,
            commands::execution::execute_custom_command,
            commands::execution::execute_command_with_custom,
            commands::execution::execute_tool_command,
            // 工具 CRUD
            commands::tools::add_tool,
            commands::tools::delete_tool,
            commands::tools::update_tool,
            commands::tools::update_tool_description,
            commands::tools::search_tools,
            commands::tools::get_all_tags,
            commands::tools::get_tool_types,
            commands::tools::get_tool_absolute_path,
            commands::tools::get_new_tools_from_scanned,
            commands::tools::auto_add_scanned_tools,
            // 分类
            commands::categories::add_category,
            commands::categories::delete_category,
            commands::categories::update_category_tools,
            commands::categories::update_category_name,
            commands::categories::update_categories_order,
            commands::categories::update_category_icon,
            // 扫描
            commands::scanning::scan_resources_for_tools,
            commands::scanning::scan_custom_directory_for_tools,
            commands::scanning::scan_tools_in_path,
            commands::scanning::scan_tools_in_custom_path,
            commands::scanning::clean_invalid_paths,
            // 笔记
            commands::notes::get_tool_note,
            commands::notes::save_tool_note,
            commands::notes::delete_tool_note,
            // 对话框
            commands::dialogs::open_file_dialog,
            commands::dialogs::open_directory_dialog,
            commands::dialogs::select_directory,
            commands::dialogs::select_file,
            commands::dialogs::select_java_path,
            commands::dialogs::select,
            // 文件浏览
            commands::files::browse_directory,
            commands::files::get_tool_directory,
            commands::files::get_file_info,
            commands::files::get_file_path,
            // 维护
            commands::misc::open_tool_directory,
            commands::misc::open_github_page,
            commands::misc::get_file_types,
            // 维护命令
            commands::maintenance_wrapper::cleanup_tool_paths_cmd,
            commands::maintenance_wrapper::repair_config_file_cmd,
            commands::maintenance_wrapper::cleanup_duplicate_tools_cmd,
            commands::maintenance_wrapper::debug_all_tool_paths_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running SpearX");
}
