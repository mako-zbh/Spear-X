# Spear-X → Tauri v2 全量 Rust 重构方案

> **文档版本**：1.0
> **日期**：2026-07-12
> **目标**：将 Spear-X 从 Wails v3 (Go) 迁移至 Tauri v2 (Rust)，全量 Rust 重写后端，前端最小改动保留

---

## 目录

- [决策摘要](#决策摘要)
- [关键发现](#关键发现)
- [一、目标目录结构](#一目标目录结构)
- [二、Cargo.toml 依赖](#二cargotoml-依赖)
- [三、Rust 数据模型](#三rust-数据模型modelsrs)
- [四、平台抽象层](#四平台抽象层platform)
- [五、完整方法→命令映射表](#五完整方法命令映射表49-个)
- [六、tauri.conf.json](#六tauriconfjson)
- [七、capabilities/main.json](#七capabilitiesmainjson)
- [八、前端改动](#八前端改动)
- [九、分阶段实施计划](#九分阶段实施计划)
- [十、风险与缓解](#十风险与缓解)
- [十一、验收标准](#十一验收标准)
- [附录：Go → Rust 概念映射速查](#附录go--rust-概念映射速查)

---

## 决策摘要

| 决策项 | 选择 | 说明 |
|---|---|---|
| 仓库结构 | 原地重构，新分支 `tauri-migration` | 在当前 Spear-X 目录内新建 `src-tauri/`，与 Go 代码共存于过渡期 |
| 前端拆分 | 不拆分 App.vue，仅替换 IPC 调用层 | 降低风险，快速验证后端；拆分留作后续技术债清理 |
| 后端方法 | 全部 49 个方法 1:1 迁移（含死代码） | 保持与 Go 后端完全对等 |
| Server Mode | 丢弃，Tauri 仅做桌面应用 | Spear-X 定位为桌面工具箱，server mode 非核心 |

---

## 关键发现

1. **`command-output` 事件是死代码**：App.vue 第 2717 行监听了它，但 Go 后端从未 emit。命令输出流向 `os.Stdout`（终端），而非 WebView。迁移时丢弃此监听器。

2. **实际生效的事件仅 4 个**：`tool-added`、`tool-updated`、`category-deleted`、`tools-scanned`，全部从 Go 后端通过 `application.Get().Event.Emit` 发出。

3. **YAML/JSON 双标签不匹配**：`Tool` 结构体的 JSON 标签使用 camelCase（如 `sourceUrl`），而 YAML 标签使用 PascalCase 且键名不同（如 `SourceURL`、`ToolName`、`PATH`、`VALUE`）。`Config` 的 Java 路径键为小写 `javapath`。serde 必须精确匹配 YAML 标签以兼容现有 `tool.yml` 配置文件。

4. **平台特定代码清单**：
   - `setHideWindow`：Windows 下设置 `CREATE_NO_WINDOW` (0x08000000) + `HideWindow`，隐藏子进程控制台窗口
   - `openTerminal`：macOS 专属，通过 AppleScript 驱动 iTerm 或 Terminal.app，非 macOS 平台返回错误
   - `getConfigDir`：3-way OS 分支（macOS `~/Library/Application Support/SpearX`、Windows `%APPDATA%/SpearX`、Linux `~/.config/spearx`）
   - `getResourcePath`：macOS `.app` bundle 路径检测（`/Contents/MacOS/` → `../Resources`）

5. **配置文件原子写入**：Go 使用 `.tmp` 临时文件 → `os.Rename` 原子替换。Rust 用 `std::fs::rename` 实现同等语义。

6. **Wails 对话框方法**（`OpenFileDialog`、`SelectDirectory` 等）在 Tauri 中可用 `tauri-plugin-dialog` 替代，但为保持 1:1 迁移仍封装为 command。

---

## 一、目标目录结构

```
Spear-X/                          # 原地重构，新分支 tauri-migration
├── src-tauri/                    # 新建：Rust 后端
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── capabilities/
│   │   └── main.json
│   ├── icons/                    # 从 build/appicon.png 生成
│   │   ├── icon.icns
│   │   ├── icon.ico
│   │   └── icon.png
│   └── src/
│       ├── main.rs               # 入口：Builder + 49个命令注册 + 窗口配置
│       ├── lib.rs                # 模块声明
│       ├── state.rs              # AppState（无字段，对齐 Go App struct{}）
│       ├── models.rs             # 全部结构体（Tool/Config/Category/JavaConfig/...）
│       ├── config.rs             # YAML 读写 + 路径解析 + 原子保存
│       ├── paths.rs              # getConfigDir / getResourcePath / getNotesDir
│       ├── executor.rs           # ExecuteCommandWithCustom 核心 + spawn_hidden
│       ├── scanner.rs            # 目录扫描 + 可执行文件检测 + analyzeToolDirectory
│       ├── notes.rs              # 笔记读写 + 迁移 + 重命名
│       ├── maintenance.rs        # RepairConfigFile / CleanupToolPaths / CleanupDuplicateTools
│       ├── platform/
│       │   ├── mod.rs            # #[cfg] 调度 + spawn_hidden 通用
│       │   ├── macos.rs          # openTerminal (AppleScript) / open_file / open_url
│       │   ├── windows.rs        # CREATE_NO_WINDOW / explorer
│       │   └── linux.rs          # xdg-open
│       └── commands/             # #[tauri::command] 层（49 个方法）
│           ├── mod.rs            # 汇总注册
│           ├── tools.rs          # AddTool/DeleteTool/UpdateTool/SearchTools/...
│           ├── categories.rs     # AddCategory/DeleteCategory/UpdateCategory*
│           ├── execution.rs      # ExecuteCommand*/ExecuteToolCommand
│           ├── scanning.rs       # Scan*/AutoAddScannedTools/CleanInvalidPaths
│           ├── notes.rs          # GetToolNote/SaveToolNote/DeleteToolNote
│           ├── config_cmd.rs     # GetJavaConfig/SaveJavaConfig/GetCategories
│           ├── dialogs.rs        # OpenFileDialog/SelectDirectory/SelectFile/SelectJavaPath
│           ├── files.rs          # BrowseDirectory/GetToolDirectory/GetFileInfo/GetFilePath
│           └── misc.rs           # OpenGitHubPage/GetToolTypes/GetFileTypes/Debug*
├── frontend/                     # 修改：替换 IPC 层
│   ├── package.json              # @wailsio/runtime → @tauri-apps/api + plugins
│   ├── vite.config.js            # 添加 Tauri 配置项
│   └── src/
│       ├── App.vue               # 仅改 import（不拆分）
│       └── api/
│           └── index.ts          # 新建：封装全部 invoke 调用
├── tool.yml                      # 保留（默认配置）
├── README.md                     # 更新构建说明
└── (Go 文件在迁移完成后删除)
```

---

## 二、Cargo.toml 依赖

```toml
[package]
name = "spearx"
version = "3.0.0"
edition = "2021"

[lib]
name = "spearx_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["macos-private-api"] }
tauri-plugin-shell = "2"
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
tauri-plugin-opener = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
chrono = { version = "0.4", features = ["serde"] }
dirs = "5"
shell-words = "1"
log = "0.4"

[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = ["Win32_Foundation", "Win32_System_Threading"] }

[profile.release]
panic = "abort"
codegen-units = 1
lto = true
opt-level = "s"
strip = true
```

**依赖说明**：

| 依赖 | 用途 | 对应 Go 依赖 |
|---|---|---|
| `tauri` | 桌面框架核心 | `wails/v3` |
| `tauri-plugin-shell` | shell 命令执行/URL 打开 | `os/exec` |
| `tauri-plugin-dialog` | 文件/目录选择对话框 | Wails Dialog API |
| `tauri-plugin-fs` | 文件系统操作 | `os` / `io/ioutil` |
| `tauri-plugin-opener` | 用系统默认程序打开文件/URL | `exec.Command("open", ...)` |
| `serde` / `serde_json` | JSON 序列化（IPC） | `encoding/json` |
| `serde_yaml` | YAML 配置读写 | `gopkg.in/yaml.v3` |
| `chrono` | 时间类型（对齐 Go `time.Time`） | `time` |
| `dirs` | 跨平台用户目录（XDG） | `adrg/xdg` |
| `shell-words` | 解析 optional 参数 | `strings.Fields` |
| `windows` (Windows only) | `CREATE_NO_WINDOW` 标志 | `syscall.SysProcAttr` |

---

## 三、Rust 数据模型（models.rs）

### 核心挑战

Go 的 `Tool` 结构体同时承载 YAML 配置和 JSON IPC 两种序列化，但两者的键名不同：

| Rust 字段 | JSON 键 (camelCase) | YAML 键 (PascalCase/混合) |
|---|---|---|
| `name` | `name` | `ToolName` |
| `path` | `path` | `PATH` |
| `file_name` | `fileName` | `FileName` |
| `value` | `value` | `VALUE` |
| `command` | `command` | `COMMAND` |
| `optional` | `optional` | `Optional` |
| `source_url` | `sourceUrl` | `SourceURL` |
| `icon_path` | `iconPath` | `IconPath` |
| `open_count` | `openCount` | `OpenCount` |

### 策略：YAML 和 JSON 分别定义结构体，通过 From/Into 互转

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// === YAML 专用结构体（精确匹配 tool.yml 键名）===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigYaml {
    #[serde(rename = "javapath")]
    pub java_paths: JavaConfig,
    #[serde(rename = "Categories")]
    pub categories: Vec<CategoryYaml>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryYaml {
    #[serde(rename = "CategoryName")]
    pub name: String,
    #[serde(rename = "Icon", default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(rename = "Tools", default)]
    pub tools: Vec<ToolYaml>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolYaml {
    #[serde(rename = "ID", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "ToolName")]
    pub name: String,
    #[serde(rename = "PATH")]
    pub path: String,
    #[serde(rename = "FileName")]
    pub file_name: String,
    #[serde(rename = "VALUE")]
    pub value: String,
    #[serde(rename = "COMMAND", default)]
    pub command: String,
    #[serde(rename = "Optional", default)]
    pub optional: String,
    #[serde(rename = "Description", default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "Tags", default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(rename = "SourceURL", default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(rename = "IconPath", default, skip_serializing_if = "Option::is_none")]
    pub icon_path: Option<String>,
    #[serde(rename = "OpenCount", default)]
    pub open_count: i32,
    #[serde(rename = "CreatedAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "LastUsedAt", default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<DateTime<Utc>>,
}

// === JSON 专用结构体（Tauri IPC，camelCase）===

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub id: Option<String>,
    pub name: String,
    pub path: String,
    pub file_name: String,
    pub value: String,
    pub command: String,
    pub optional: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub source_url: Option<String>,
    pub icon_path: Option<String>,
    pub open_count: i32,
    pub created_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub name: String,
    pub icon: Option<String>,
    pub tools: Vec<Tool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Categories {
    pub categories: Vec<Category>,
}

// JavaConfig 特殊：前端期望 PascalCase 键 {Java8, Java11, Java17}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct JavaConfig {
    pub java8: String,
    pub java11: String,
    pub java17: String,
}

// === 其他结构体 ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedTool {
    pub path: String,
    pub category: String,
    pub possible_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub name: String,
    pub is_dir: bool,
    pub size: i64,
    pub mod_time: String,
    pub path: String,
    pub extension: String,
    pub is_executable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupResult {
    pub invalid_tools_count: i32,
    pub invalid_categories_count: i32,
    pub cleaned_notes: i32,
    pub migrated_notes: i32,
    pub invalid_tool_names: Vec<String>,
    pub migrated_tool_names: Vec<String>,
}

// === From/Into 互转实现 ===

impl From<ToolYaml> for Tool {
    fn from(y: ToolYaml) -> Self {
        Tool {
            id: y.id,
            name: y.name,
            path: y.path,
            file_name: y.file_name,
            value: y.value,
            command: y.command,
            optional: y.optional,
            description: y.description,
            tags: y.tags,
            source_url: y.source_url,
            icon_path: y.icon_path,
            open_count: y.open_count,
            created_at: y.created_at,
            last_used_at: y.last_used_at,
        }
    }
}

impl From<Tool> for ToolYaml {
    fn from(t: Tool) -> Self {
        ToolYaml {
            id: t.id,
            name: t.name,
            path: t.path,
            file_name: t.file_name,
            value: t.value,
            command: t.command,
            optional: t.optional,
            description: t.description,
            tags: t.tags,
            source_url: t.source_url,
            icon_path: t.icon_path,
            open_count: t.open_count,
            created_at: t.created_at,
            last_used_at: t.last_used_at,
        }
    }
}

// CategoryYaml <-> Category、ConfigYaml <-> Categories 的 From/Into 同理
```

### 设计要点

1. **YAML 结构体**：每个字段的 `rename` 精确匹配 `tool.yml` 中的键名，确保现有配置文件无损读取
2. **JSON 结构体**：使用 `rename_all = "camelCase"`，使 Tauri IPC 返回的前端数据格式与原 Wails 一致
3. **`JavaConfig` 特殊处理**：前端代码直接访问 `config.Java8`、`config.Java11`、`config.Java17`（PascalCase），因此使用 `rename_all = "PascalCase"`
4. **`omitempty` 语义**：Go 的 `yaml:"...,omitempty"` 在 Rust 中用 `default` + `skip_serializing_if = "Option::is_none"`（可选字段）或 `skip_serializing_if = "Vec::is_empty"`（空数组）实现
5. **`time.Time` 映射**：Go 的 `time.Time` 在 YAML 中序列化为 RFC3339 格式。Rust 用 `Option<DateTime<Utc>>`，`chrono` 的 serde 特性自动处理 RFC3339 格式

---

## 四、平台抽象层（platform/）

### platform/mod.rs — 通用函数 + 平台调度

```rust
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
pub fn open_in_file_manager(path: &str) -> std::io::Result<()> {
    let mut cmd = match std::env::consts::OS {
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
    spawn_hidden(cmd)
}

/// 用系统默认程序打开 URL（对齐 Go Browser case + OpenGitHubPage）
///
/// macOS: open, Windows: cmd /C start, Linux: xdg-open
pub fn open_url(url: &str) -> std::io::Result<()> {
    let mut cmd = match std::env::consts::OS {
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
    spawn_hidden(cmd)
}
```

### platform/macos.rs — AppleScript 终端启动

对齐 Go `app.go:369` 的 `openTerminal` 函数：

```rust
use std::process::Command;
use std::path::Path;

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
```

### platform/windows.rs + platform/linux.rs

```rust
// platform/windows.rs
use std::path::Path;

/// 非 macOS 平台，对齐 Go 的 "不支持的平台" 错误
pub fn open_terminal(_dir: &Path, _cmd: Option<&str>) -> Result<(), String> {
    // Go 原始代码在非 macOS 平台返回 "不支持的平台" 错误
    // 后续可选：实现 Windows Terminal / xterm 启动
    Err("不支持的平台".to_string())
}

// platform/linux.rs — 同上
pub fn open_terminal(_dir: &Path, _cmd: Option<&str>) -> Result<(), String> {
    Err("不支持的平台".to_string())
}
```

---

## 五、完整方法→命令映射表（49 个）

### 5.1 生命周期

| Go 方法 | Tauri 实现 | 说明 |
|---|---|---|
| `ServiceStartup` | `main.rs` 的 `setup()` 闭包 | 调用 `ensure_config_exists()`，异步 spawn `repair_config_file()` + `cleanup_tool_paths()` |

### 5.2 配置（config_cmd.rs + config.rs）

| Go 方法 | Tauri command | 参数 | 返回类型 |
|---|---|---|---|
| `GetCategories()` | `get_categories` | 无 | `Categories` |
| `GetJavaConfig()` | `get_java_config` | 无 | `JavaConfig` |
| `SaveJavaConfig(config)` | `save_java_config` | `config: JavaConfig` | `()` |

### 5.3 执行引擎（execution.rs + executor.rs）

| Go 方法 | Tauri command | 参数 |
|---|---|---|
| `ExecuteCommand(path, optional, value, filename)` | `execute_command` | 4 个 String |
| `ExecuteCustomCommand(path, optional, value, filename, customCommand)` | `execute_custom_command` | 5 个 String |
| `ExecuteCommandWithCustom(path, optional, value, filename, customCommand, javaPath)` | `execute_command_with_custom` | 6 个 String |
| `ExecuteToolCommand(tool, customCommand)` | `execute_tool_command` | `tool: Tool, customCommand: String` |

**执行引擎核心逻辑**（`executor.rs`，对齐 Go `app.go:217` `ExecuteCommandWithCustom`）：

```
switch value:
  case "Java8" | "Java11" | "Java17":
    java_exec = config.java_paths.{对应版本}，为空则回退 "java"
    jar_path = tool_path / filename
    args = [optional 解析后的参数...] + ["-jar", filename]
    spawn_hidden(Command::new(java_exec).args(args).current_dir(tool_path))

  case "Open":
    spawn_hidden(Command::new("open").arg(filename).current_dir(tool_path))  // macOS

  case "openterm":
    if custom_command 非空:
      替换 {file} -> tool_path/filename, {filename} -> filename, {path} -> tool_path
      open_terminal(tool_path, Some(替换后的命令))
    else:
      open_terminal(tool_path, None)

  case "Browser":
    if path 以 http:// 或 https:// 开头:
      target = path
    else if filename 非空:
      target = tool_path / filename
    else:
      target = tool_path
    open_url(target)

  case "Binary":
    binary_path = tool_path / filename
    spawn_hidden(Command::new(binary_path).current_dir(tool_path))

  default:
    return Err("不支持的命令类型: {value}")
```

### 5.4 工具 CRUD（tools.rs）

| Go 方法 | Tauri command | 参数 |
|---|---|---|
| `AddTool(tool, categoryName)` | `add_tool` | `tool: Tool, categoryName: String` |
| `DeleteTool(toolName, categoryName)` | `delete_tool` | `toolName: String, categoryName: String` |
| `UpdateTool(originalName, categoryName, tool)` | `update_tool` | `originalName: String, categoryName: String, tool: Tool` |
| `UpdateToolDescription(toolName, categoryName, description)` | `update_tool_description` | 3 个 String |
| `SearchTools(query)` | `search_tools` | `query: String` |
| `GetAllTags()` | `get_all_tags` | 无 |
| `GetToolTypes()` | `get_tool_types` | 无 |
| `GetToolAbsolutePath(toolPath, fileName)` | `get_tool_absolute_path` | 2 个 String |
| `GetNewToolsFromScanned(tools)` | `get_new_tools_from_scanned` | `tools: Vec<ScannedTool>` |
| `AutoAddScannedTools(tools)` | `auto_add_scanned_tools` | `tools: Vec<ScannedTool>` |

### 5.5 分类（categories.rs）

| Go 方法 | Tauri command | 参数 |
|---|---|---|
| `AddCategory(categoryName)` | `add_category` | `categoryName: String` |
| `DeleteCategory(categoryName)` | `delete_category` | `categoryName: String` |
| `UpdateCategoryTools(categoryName, tools)` | `update_category_tools` | `categoryName: String, tools: Vec<Tool>` |
| `UpdateCategoryName(oldName, newName)` | `update_category_name` | 2 个 String |
| `UpdateCategoriesOrder(orderedCategories)` | `update_categories_order` | `orderedCategories: Vec<Category>` |
| `UpdateCategoryIcon(categoryName, icon)` | `update_category_icon` | 2 个 String |

### 5.6 扫描（scanning.rs + scanner.rs）

| Go 方法 | Tauri command | 参数 |
|---|---|---|
| `ScanResourcesForTools()` | `scan_resources_for_tools` | 无 |
| `ScanCustomDirectoryForTools(customPath)` | `scan_custom_directory_for_tools` | `customPath: String` |
| `ScanToolsInPath(scanPath)` | `scan_tools_in_path` | `scanPath: String` |
| `ScanToolsInCustomPath(scanPath)` | `scan_tools_in_custom_path` | `scanPath: String` |
| `CleanInvalidPaths()` | `clean_invalid_paths` | 无 |

**扫描逻辑要点**（`scanner.rs`，对齐 Go `app.go:1400`）：

- `ScanToolsInPath`：遍历 `<scanPath>/<categoryDir>/<toolDir>`，跳过 `java8/java11/java17` 目录，构建 `ScannedTool`（path 为 `resources/...` 相对路径）
- `ScanToolsInCustomPath`：两种模式——目录嵌套（dir-of-dirs，用目录名作分类）和扁平（分类为 "自定义工具"），使用绝对路径
- `analyzeToolDirectory`：优先级 jar > binary > .app > openterm，推断工具类型和文件名
- `isExecutableFile`：检查扩展名 `.jar/.exe/.app/.sh/.py/.bat/.cmd` + Unix 执行权限位（`mode & 0o111`）
- `isBinaryExecutable`：无扩展名文件，排除 readme/license/changelog/makefile 等

### 5.7 笔记（notes.rs）

| Go 方法 | Tauri command | 参数 |
|---|---|---|
| `GetToolNote(toolPath, toolName)` | `get_tool_note` | 2 个 String |
| `SaveToolNote(toolPath, toolName, content)` | `save_tool_note` | 3 个 String |
| `DeleteToolNote(toolPath, toolName)` | `delete_tool_note` | 2 个 String |

**笔记存储逻辑**（对齐 Go `app.go:1174`）：

- 当前方案：笔记文件为 `{toolDir}/{toolName}.md`，`toolDir` 为工具绝对路径
- 遗留方案：`{notesDir}/{id}.md`（id 由路径末段 + 空格→下划线 + 连字符→下划线 派生）
- 读取时：若当前方案文件不存在，尝试从遗留位置迁移
- 重命名工具时：`{oldName}.md` → `{newName}.md`，若目标已存在则创建 `{newName}_backup_{timestamp}.md`

### 5.8 对话框（dialogs.rs）

| Go 方法 | Tauri command | 实现方式 |
|---|---|---|
| `OpenFileDialog()` | `open_file_dialog` | `tauri-plugin-dialog` 的 `open` + `GetFileInfo` 逻辑 |
| `OpenDirectoryDialog()` | `open_directory_dialog` | `tauri-plugin-dialog` 的 `open`（CanChooseDirectories） |
| `SelectDirectory()` | `select_directory` | `tauri-plugin-dialog`，标题 "选择要扫描的文件夹" |
| `SelectFile()` | `select_file` | `tauri-plugin-dialog`，标题 "选择文件" |
| `SelectJavaPath()` | `select_java_path` | `tauri-plugin-dialog`，标题 "选择Java可执行文件" |
| `Select(selectFolder)` | `select` | 复刻 `/Applications/Spear.app/Contents/Resources` 目录限制逻辑 |

### 5.9 文件浏览（files.rs）

| Go 方法 | Tauri command | 参数 | 返回 |
|---|---|---|---|
| `BrowseDirectory(pathInput)` | `browse_directory` | `pathInput: String` | `Vec<FileInfo>` |
| `GetToolDirectory(toolPath)` | `get_tool_directory` | `toolPath: String` | `Vec<FileInfo>` |
| `GetFileInfo(filePath)` | `get_file_info` | `filePath: String` | `HashMap<String, String>` |
| `GetFilePath(fileName)` | `get_file_path` | `fileName: String` | `String` |

### 5.10 维护（maintenance.rs）

| Go 方法 | Tauri command | 说明 |
|---|---|---|
| `CleanupToolPaths()` | `cleanup_tool_paths` | 对所有工具运行 `clean_tool_path`，有变更则保存 |
| `RepairConfigFile()` | `repair_config_file` | 验证失败则从 `.backup` 恢复，备份也坏则写默认空配置 |
| `CleanupDuplicateTools()` | `cleanup_duplicate_tools` | 合并重复工具（同 Path），优先保留中文分类名 |
| `DebugAllToolPaths()` | `debug_all_tool_paths` | 输出所有工具路径用于调试 |

### 5.11 杂项（misc.rs）

| Go 方法 | Tauri command | 说明 |
|---|---|---|
| `OpenToolDirectory(path)` | `open_tool_directory` | 调用 `platform::open_in_file_manager` |
| `OpenGitHubPage()` | `open_github_page` | 调用 `platform::open_url` 打开 GitHub |
| `GetFileTypes()` | `get_file_types` | 返回静态工具类型列表 |

### 5.12 事件 emit 映射

Go `application.Get().Event.Emit("xxx", true)` → Rust `app_handle.emit("xxx", true)`：

| Go emit 位置 | 事件名 | 触发场景 |
|---|---|---|
| `app.go:497` | `tool-added` | `AddTool` 完成后 |
| `app.go:925` | `tool-updated` | `UpdateTool` 完成后 |
| `app.go:1011` | `category-deleted` | `DeleteCategory` 完成后 |
| `app.go:1049` | `tool-updated` | `UpdateCategoryTools` 完成后 |
| `app.go:1101` | `tool-updated` | `UpdateToolDescription` 完成后 |
| `app.go:2189` | `tools-scanned` | `saveCategoriesToFile` 完成后 |

> **`command-output` 事件：丢弃。** Go 后端从未 emit 此事件，App.vue 的监听器是死代码。命令输出实际流向 `os.Stdout`（终端），不进入 WebView。

### 5.13 main.rs 命令注册

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .setup(|app| {
            let state = app.state::<AppState>();
            state.ensure_config_exists()?;
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = handle.state::<AppState>();
                let _ = state.repair_config_file();
                let _ = state.cleanup_tool_paths();
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
            commands::maintenance::cleanup_tool_paths,
            commands::maintenance::repair_config_file,
            commands::maintenance::cleanup_duplicate_tools,
            commands::maintenance::debug_all_tool_paths,
            // 杂项
            commands::misc::open_tool_directory,
            commands::misc::open_github_page,
            commands::misc::get_file_types,
        ])
        .run(tauri::generate_context!())
        .expect("error while running SpearX");
}
```

---

## 六、tauri.conf.json

```json
{
  "$schema": "https://raw.githubusercontent.com/nicehash/tauri/dev/crates/tauri-utils/schema.json",
  "productName": "SpearX",
  "version": "3.0.0",
  "identifier": "com.spe4r.spearx",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:9245",
    "frontendDist": "../frontend/dist"
  },
  "app": {
    "windows": [
      {
        "title": "SpearX",
        "width": 1024,
        "height": 768,
        "decorations": false,
        "transparent": true,
        "titleBarStyle": "Overlay",
        "hiddenTitle": true
      }
    ],
    "security": {
      "csp": null
    },
    "macOSPrivateApi": true
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/icon.png",
      "icons/icon.ico",
      "icons/icon.icns"
    ],
    "resources": ["../tool.yml"],
    "macOS": {
      "minimumSystemVersion": "12.0.0"
    },
    "windows": {
      "nsis": {
        "installerIcon": "icons/icon.ico"
      }
    }
  }
}
```

**配置说明**：

| 配置项 | 值 | 对应 Go/Wails |
|---|---|---|
| `productName` | `SpearX` | `build/config.yml` info.productName |
| `version` | `3.0.0` | 升级版本号 |
| `identifier` | `com.spe4r.spearx` | `build/config.yml` info.productIdentifier |
| `build.devUrl` | `http://localhost:9245` | Taskfile VITE_PORT 9245 |
| `build.frontendDist` | `../frontend/dist` | Go `//go:embed all:frontend/dist` |
| `windows.decorations` | `false` | macOS transparent titlebar |
| `windows.transparent` | `true` | macOS NSVisualEffectView |
| `windows.titleBarStyle` | `Overlay` | macOS full-size content |
| `macOSPrivateApi` | `true` | 启用原生毛玻璃 |
| `macOS.minimumSystemVersion` | `12.0.0` | Info.plist LSMinimumSystemVersion |

> **窗口毛玻璃**：macOS `transparent: true` + `macOSPrivateApi: true` + `titleBarStyle: "Overlay"` 对齐 Go 的 NSVisualEffectView 效果。Windows/Linux 仍用 CSS 模拟（`main.css` 不变）。可后续集成 `window-vibrancy` crate 获得原生效果。

---

## 七、capabilities/main.json

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "main-capability",
  "description": "SpearX 主窗口权限",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-execute",
    "shell:allow-open",
    "dialog:allow-open",
    "fs:allow-read-file",
    "fs:allow-write-file",
    "opener:default"
  ]
}
```

**安全策略说明**：

Spear-X 的执行引擎需要调用任意外部程序（`java`、`explorer`、`osascript` 等），这与 Tauri shell scope 的严格限制存在张力。

**方案**：执行引擎不使用 `tauri-plugin-shell` 的 `execute` API，而是直接在 `#[tauri::command]` 内使用 `std::process::Command`。这样绕过了 shell scope 的 URL/命令白名单限制，同时保持了代码的灵活性。`tauri-plugin-shell` 仅保留给 `OpenGitHubPage` 等简单场景使用。`dialog` 和 `fs` 插件正常使用其权限系统。

---

## 八、前端改动

### 8.1 package.json

```diff
 "dependencies": {
   "@element-plus/icons-vue": "^2.3.1",
-  "@wailsio/runtime": "^3.0.0-alpha.97",
+  "@tauri-apps/api": "^2",
+  "@tauri-apps/plugin-shell": "^2",
+  "@tauri-apps/plugin-dialog": "^2",
+  "@tauri-apps/plugin-fs": "^2",
   "element-plus": "^2.5.6",
   "vue": "^3.4.15",
   "vuedraggable": "^4.1.0"
 },
```

### 8.2 vite.config.js

```js
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  // Tauri 配置
  clearScreen: false,
  server: {
    port: 9245,
    strictPort: true,
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: ['es2021', 'chrome100', 'safari13'],
    minify: 'esbuild',
    sourcemap: false,
  },
})
```

### 8.3 新建 frontend/src/api/index.ts

封装全部 49 个 `invoke()` 调用 + TypeScript 类型定义：

```typescript
import { invoke } from '@tauri-apps/api/core';

// === 类型定义（对齐 Rust serde JSON 输出）===

export interface Tool {
  id?: string;
  name: string;
  path: string;
  fileName: string;
  value: string;
  command: string;
  optional: string;
  description?: string;
  tags: string[];
  sourceUrl?: string;
  iconPath?: string;
  openCount: number;
  createdAt?: string;
  lastUsedAt?: string;
}

export interface Category {
  name: string;
  icon?: string;
  tools: Tool[];
}

export interface Categories {
  categories: Category[];
}

export interface JavaConfig {
  Java8: string;
  Java11: string;
  Java17: string;
}

export interface ScannedTool {
  path: string;
  category: string;
  possibleFiles: string[];
}

export interface FileInfo {
  name: string;
  isDir: boolean;
  size: number;
  modTime: string;
  path: string;
  extension: string;
  isExecutable: boolean;
}

export interface CleanupResult {
  invalidToolsCount: number;
  invalidCategoriesCount: number;
  cleanedNotes: number;
  migratedNotes: number;
  invalidToolNames: string[];
  migratedToolNames: string[];
}

// === 配置 ===

export const getCategories = () => invoke<Categories>('get_categories');
export const getJavaConfig = () => invoke<JavaConfig | null>('get_java_config');
export const saveJavaConfig = (config: JavaConfig) =>
  invoke('save_java_config', { config });

// === 执行 ===

export const executeCommand = (
  path: string, optional: string, value: string, filename: string
) => invoke('execute_command', { path, optional, value, filename });

export const executeCustomCommand = (
  path: string, optional: string, value: string, filename: string, customCommand: string
) => invoke('execute_custom_command', { path, optional, value, filename, customCommand });

export const executeCommandWithCustom = (
  path: string, optional: string, value: string, filename: string,
  customCommand: string, javaPath: string
) => invoke('execute_command_with_custom', {
  path, optional, value, filename, customCommand, javaPath
});

export const executeToolCommand = (tool: Tool, customCommand: string) =>
  invoke('execute_tool_command', { tool, customCommand });

// === 工具 CRUD ===

export const addTool = (tool: Tool, categoryName: string) =>
  invoke('add_tool', { tool, categoryName });

export const deleteTool = (toolName: string, categoryName: string) =>
  invoke('delete_tool', { toolName, categoryName });

export const updateTool = (originalName: string, categoryName: string, tool: Tool) =>
  invoke('update_tool', { originalName, categoryName, tool });

export const updateToolDescription = (
  toolName: string, categoryName: string, description: string
) => invoke('update_tool_description', { toolName, categoryName, description });

export const searchTools = (query: string) =>
  invoke<Tool[]>('search_tools', { query });

export const getAllTags = () => invoke<string[]>('get_all_tags');

export const getToolTypes = () => invoke<string[]>('get_tool_types');

export const getToolAbsolutePath = (toolPath: string, fileName: string) =>
  invoke<string>('get_tool_absolute_path', { toolPath, fileName });

export const getNewToolsFromScanned = (tools: ScannedTool[]) =>
  invoke<ScannedTool[]>('get_new_tools_from_scanned', { tools });

export const autoAddScannedTools = (tools: ScannedTool[]) =>
  invoke('auto_add_scanned_tools', { tools });

// === 分类 ===

export const addCategory = (categoryName: string) =>
  invoke('add_category', { categoryName });

export const deleteCategory = (categoryName: string) =>
  invoke('delete_category', { categoryName });

export const updateCategoryTools = (categoryName: string, tools: Tool[]) =>
  invoke('update_category_tools', { categoryName, tools });

export const updateCategoryName = (oldName: string, newName: string) =>
  invoke('update_category_name', { oldName, newName });

export const updateCategoriesOrder = (orderedCategories: Category[]) =>
  invoke('update_categories_order', { orderedCategories });

export const updateCategoryIcon = (categoryName: string, icon: string) =>
  invoke('update_category_icon', { categoryName, icon });

// === 扫描 ===

export const scanResourcesForTools = () =>
  invoke<ScannedTool[]>('scan_resources_for_tools');

export const scanCustomDirectoryForTools = (customPath: string) =>
  invoke<ScannedTool[]>('scan_custom_directory_for_tools', { customPath });

export const scanToolsInPath = (scanPath: string) =>
  invoke<ScannedTool[]>('scan_tools_in_path', { scanPath });

export const scanToolsInCustomPath = (scanPath: string) =>
  invoke<ScannedTool[]>('scan_tools_in_custom_path', { scanPath });

export const cleanInvalidPaths = () =>
  invoke<CleanupResult>('clean_invalid_paths');

// === 笔记 ===

export const getToolNote = (toolPath: string, toolName: string) =>
  invoke<string>('get_tool_note', { toolPath, toolName });

export const saveToolNote = (
  toolPath: string, toolName: string, content: string
) => invoke('save_tool_note', { toolPath, toolName, content });

export const deleteToolNote = (toolPath: string, toolName: string) =>
  invoke('delete_tool_note', { toolPath, toolName });

// === 对话框 ===

export const openFileDialog = () =>
  invoke<Record<string, string>>('open_file_dialog');

export const openDirectoryDialog = () =>
  invoke<string>('open_directory_dialog');

export const selectDirectory = () =>
  invoke<string>('select_directory');

export const selectFile = () =>
  invoke<string>('select_file');

export const selectJavaPath = () =>
  invoke<string>('select_java_path');

export const select = (selectFolder: boolean) =>
  invoke<string>('select', { selectFolder });

// === 文件浏览 ===

export const browseDirectory = (pathInput: string) =>
  invoke<FileInfo[]>('browse_directory', { pathInput });

export const getToolDirectory = (toolPath: string) =>
  invoke<FileInfo[]>('get_tool_directory', { toolPath });

export const getFileInfo = (filePath: string) =>
  invoke<Record<string, string>>('get_file_info', { filePath });

export const getFilePath = (fileName: string) =>
  invoke<string>('get_file_path', { fileName });

// === 维护 ===

export const cleanupToolPaths = () => invoke('cleanup_tool_paths');
export const repairConfigFile = () => invoke('repair_config_file');
export const cleanupDuplicateTools = () => invoke('cleanup_duplicate_tools');
export const debugAllToolPaths = () => invoke('debug_all_tool_paths');

// === 杂项 ===

export const openToolDirectory = (path: string) =>
  invoke('open_tool_directory', { path });

export const openGitHubPage = () => invoke('open_github_page');

export const getFileTypes = () =>
  invoke<Record<string, string>[]>('get_file_types');
```

### 8.4 App.vue 改动点（最小化，不拆分）

**改动 1：替换 import（第 735-770 行）**

```diff
- import { Events } from '@wailsio/runtime'
- import {
-   AddCategory, AddTool, AutoAddScannedTools, CleanInvalidPaths, CleanupToolPaths,
-   DebugAllToolPaths, DeleteCategory, DeleteTool, DeleteToolNote,
-   ExecuteCommand, ExecuteCustomCommand, GetAllTags, GetCategories, GetJavaConfig,
-   GetNewToolsFromScanned, GetToolAbsolutePath, GetToolDirectory, GetToolNote,
-   GetToolTypes, OpenGitHubPage, OpenToolDirectory, SaveJavaConfig, SaveToolNote,
-   ScanCustomDirectoryForTools, ScanResourcesForTools,
-   SelectDirectory, SelectFile, SelectJavaPath,
-   UpdateCategoriesOrder, UpdateCategoryIcon, UpdateCategoryName,
-   UpdateCategoryTools, UpdateTool
- } from '../bindings/SSPSecTools/app.js'
+ import { listen } from '@tauri-apps/api/event'
+ import * as api from './api'
```

**改动 2：替换全部约 40 个调用点**

所有后端方法调用从 PascalCase 改为 `api.camelCase`：

| 原调用 | 新调用 |
|---|---|
| `GetCategories()` | `api.getCategories()` |
| `AddTool(tool, category)` | `api.addTool(tool, category)` |
| `ExecuteCustomCommand(...)` | `api.executeCustomCommand(...)` |
| `SaveJavaConfig(config)` | `api.saveJavaConfig(config)` |
| `GetToolNote(toolPath, toolName)` | `api.getToolNote(toolPath, toolName)` |
| ... | ...（全部 40 处同理） |

**改动 3：替换事件监听（第 2716-2735 行）**

```diff
  onMounted(async () => {
    // 事件监听
-   const cancelCommandOutput = Events.On('command-output', (event) => {
-     outputText.value = event.data;
-   });
-   const cancelToolAdded = Events.On('tool-added', () => {
+   const unlistenToolAdded = await listen('tool-added', () => {
      loadCategories();
      loadAllTags();
      showAddDialog.value = false;
      ElMessage.success('工具添加成功');
    });
-   const cancelToolUpdated = Events.On('tool-updated', () => {
+   const unlistenToolUpdated = await listen('tool-updated', () => {
      if (silentUpdate.value) return;
      loadCategories();
      loadAllTags();
      editDialog.visible = false;
    });
    // ...
  });

  onBeforeUnmount(() => {
-   cancelCommandOutput();
-   cancelToolAdded();
-   cancelToolUpdated();
+   unlistenToolAdded();
+   unlistenToolUpdated();
  });
```

> `command-output` 监听器删除（死代码，Go 后端从未 emit）。

**改动 4：删除 frontend/bindings/ 目录**

Wails 自动生成的绑定文件不再需要。

---

## 九、分阶段实施计划

### Phase 0：脚手架搭建（1 天）

- [ ] 创建 `tauri-migration` 分支：`git checkout -b tauri-migration`
- [ ] 安装 Rust toolchain：`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- [ ] 安装 Tauri CLI：`cargo install tauri-cli --version "^2.0.0"`
- [ ] 初始化 Tauri 项目：在项目根目录运行 `cargo tauri init`，生成 `src-tauri/` 骨架
- [ ] 配置 `Cargo.toml`（依赖列表见第二节）
- [ ] 配置 `tauri.conf.json`（见第六节）
- [ ] 配置 `capabilities/main.json`（见第七节）
- [ ] 生成图标：`cargo tauri icon build/appicon.png`
- [ ] 修改 `frontend/package.json`（见第 8.1 节）
- [ ] 修改 `frontend/vite.config.js`（见第 8.2 节）
- [ ] 运行 `npm install`（在 frontend/ 目录）
- [ ] 验证空壳 Tauri + Vue 能启动：`cargo tauri dev`（应显示空白窗口）

### Phase 1：数据层（2 天）

- [ ] `models.rs`：全部结构体
  - YAML 专用：`ConfigYaml`、`CategoryYaml`、`ToolYaml`
  - JSON 专用：`Tool`、`Category`、`Categories`、`JavaConfig`
  - 其他：`ScannedTool`、`FileInfo`、`CleanupResult`
  - `From`/`Into` 互转实现
- [ ] `paths.rs`：
  - `get_config_dir()`：3-way OS 分支（macOS/Windows/Linux）
  - `get_config_path()`：`config_dir/tool.yml`
  - `get_notes_dir()`：`config_dir/notes`
  - `get_resource_path()`：`.app` bundle 检测 + dev 模式回退
- [ ] `config.rs`：
  - `load_config()`：读取 `tool.yml`，反序列化为 `ConfigYaml`
  - `save_config()`：序列化 `ConfigYaml`，写入 `.tmp` → `rename`（原子写入）
  - `ensure_config_exists()`：首次运行创建默认配置，从旧路径迁移
  - `validate_config_file()`：检查 YAML 格式 + `Categories:` 和 `javapath:` 键存在
- [ ] **验证**：能正确读写现有 `tool.yml`，YAML 标签精确匹配，无数据丢失

### Phase 2：执行引擎 + 平台层（2 天）★ 关键路径

- [ ] `platform/mod.rs`：
  - `spawn_hidden(cmd)`：Windows 下 `CREATE_NO_WINDOW`
  - `open_in_file_manager(path)`：open/explorer/xdg-open
  - `open_url(url)`：open/cmd start/xdg-open
- [ ] `platform/macos.rs`：
  - `open_terminal(dir, initial_command)`：AppleScript，iTerm/Terminal.app 分支
- [ ] `platform/windows.rs` + `platform/linux.rs`：
  - `open_terminal()` 返回 "不支持的平台"
- [ ] `executor.rs`：
  - `execute_command_with_custom(path, optional, value, filename, custom_command, java_path)`
  - switch on `value`：Java8/11/17、Open、openterm、Browser、Binary
  - 变量替换：`{file}`、`{filename}`、`{path}` → 实际值
- [ ] 注册 4 个执行 command 到 `main.rs`
- [ ] **验证**：macOS 上 Java jar / Open / openterm / Browser / Binary 五种类型都能启动

### Phase 3：命令层 - 配置与 CRUD（2 天）

- [ ] `commands/config_cmd.rs`：`get_categories`、`get_java_config`、`save_java_config`
- [ ] `commands/tools.rs`：10 个工具方法（add/delete/update/search/tags/types/...）
- [ ] `commands/categories.rs`：6 个分类方法
- [ ] `state.rs`：`AppState` 结构体 + 事件 emit 封装（`app_handle.emit`）
- [ ] `main.rs`：注册命令 + setup hook（ensure_config + async repair/cleanup）
- [ ] **验证**：前端能调用 CRUD，`tool-added`/`tool-updated`/`category-deleted` 事件正确 emit

### Phase 4：命令层 - 扫描与笔记（2 天）

- [ ] `scanner.rs`：
  - `scan_tools_in_path()`：遍历目录，跳过 java 目录
  - `scan_tools_in_custom_path()`：两种模式（嵌套/扁平）
  - `analyze_tool_directory()`：优先级 jar > binary > .app > openterm
  - `is_executable_file()` / `is_binary_executable()`
- [ ] `commands/scanning.rs`：5 个扫描 + `get_new_tools_from_scanned` + `auto_add_scanned_tools`
- [ ] `notes.rs`：
  - `get_tool_note()` / `save_tool_note()` / `delete_tool_note()`
  - `rename_tool_note()`：重命名时备份
  - 遗留笔记迁移逻辑
- [ ] `commands/notes.rs`：3 个笔记命令
- [ ] **验证**：目录扫描能发现工具，笔记读写正常，迁移逻辑正确

### Phase 5：命令层 - 对话框/文件/维护/杂项（1 天）

- [ ] `commands/dialogs.rs`：6 个对话框方法（使用 `tauri-plugin-dialog`）
- [ ] `commands/files.rs`：4 个文件浏览方法
- [ ] `maintenance.rs`：`repair_config_file`、`cleanup_tool_paths`、`cleanup_duplicate_tools`、`debug_all_tool_paths`
- [ ] `commands/misc.rs`：`open_tool_directory`、`open_github_page`、`get_file_types`
- [ ] `main.rs`：注册全部 49 个命令（清单见第 5.13 节）
- [ ] **验证**：`generate_handler!` 宏中命令列表无遗漏，`cargo check` 通过

### Phase 6：前端适配（2 天）

- [ ] 新建 `frontend/src/api/index.ts`（49 个方法封装，见第 8.3 节）
- [ ] 修改 `App.vue`：
  - 替换 import（删除 Wails bindings，改用 `api` 模块）
  - 替换全部约 40 个调用点（PascalCase → `api.camelCase`）
  - 替换事件监听（`Events.On` → `await listen(...)`，删除 `command-output` 死监听）
  - `onMounted` 改为 `async`
- [ ] 删除 `frontend/bindings/` 目录
- [ ] **验证**：`cargo tauri dev` 全流程跑通，所有功能可用

### Phase 7：构建与分发（1 天）

- [ ] 验证 `cargo tauri build` 产出 `.app`（macOS）
- [ ] 验证 NSIS `.exe`（Windows，如有环境）
- [ ] 验证 `.deb`/`.AppImage`（Linux，如有环境）
- [ ] 配置 GitHub Actions CI（`tauri-apps/tauri-action`）
- [ ] 更新 `README.md`（构建说明：`cargo tauri dev` / `cargo tauri build`）

### Phase 8：清理（0.5 天）

- [ ] 删除 Go 源文件：`app.go`、`main.go`、`hide_window_unix.go`、`hide_window_windows.go`、`go.mod`、`go.sum`
- [ ] 删除 Wails 构建文件：`build/` 目录（保留 `appicon.png` 引用）、`Taskfile.yml`
- [ ] 删除预编译二进制：`SSPSecTools`、`bin/`
- [ ] 最终全功能回归测试

**总预估：13-14 个工作日**

### 甘特图

```
Phase 0  ██████                          (1d)  脚手架
Phase 1  ████████████                    (2d)  数据层
Phase 2  ████████████                    (2d)  执行引擎 ★
Phase 3  ████████████                    (2d)  配置+CRUD
Phase 4  ████████████                    (2d)  扫描+笔记
Phase 5  ██████                          (1d)  对话框+维护
Phase 6  ████████████                    (2d)  前端适配
Phase 7  ██████                          (1d)  构建+CI
Phase 8  ███                             (0.5d)清理
         ─────────────────────────────────────
         总计 13.5 工作日
```

---

## 十、风险与缓解

| 风险 | 影响 | 缓解措施 |
|---|---|---|
| serde YAML/JSON 双标签冲突 | 序列化/反序列化失败 | YAML 和 JSON 分别定义结构体 + `From`/`Into` 互转，彻底隔离两套标签 |
| Tauri shell scope 限制执行引擎 | 无法执行任意外部程序 | 执行引擎直接用 `std::process::Command`，不走 shell 插件 scope |
| AppleScript 转义问题（路径含引号） | 脚本注入或执行失败 | Rust 中对路径做单引号转义：`replace('\'', "'\\''")` |
| `time.Time` omitempty 语义差异 | 配置文件字段多余或缺失 | Rust 用 `Option<DateTime<Utc>>` + `skip_serializing_if = "Option::is_none"` |
| 前端 `GetToolNote(tool.name)` 单参数 bug | 笔记读取异常 | 保持 1:1 行为：Rust command 接收 `toolPath` 和 `toolName` 两个参数，`toolName` 为空时做容错退化 |
| macOS 毛玻璃效果不理想 | UI 体验下降 | `transparent` + `macOSPrivateApi`；如效果不理想集成 `window-vibrancy` crate 获得原生 NSVisualEffectView |
| Rust 编译时间（首次约 5 分钟） | 开发体验下降 | 开发期用 `cargo tauri dev`（增量编译，约 30s）；release 用 LTO + strip 优化体积 |
| 配置文件迁移兼容性 | 用户现有 `tool.yml` 无法读取 | Phase 1 完成后立即用真实 `tool.yml` 验证读写，确保 YAML 标签精确匹配 |
| `getPossibleDirNames` 中文映射 | 扫描分类不正确 | 精确复刻 Go 中的硬编码映射表（信息收集→info/information/recon 等） |

---

## 十一、验收标准

### 功能验收

1. **工具执行**：Java8/11/17（jar）、Open、openterm（macOS AppleScript）、Browser、Binary 五种类型全部可正常启动
2. **工具管理**：添加、删除、编辑工具功能正常，工具名称唯一性校验生效
3. **分类管理**：添加、删除、重命名分类，拖拽排序，图标设置功能正常
4. **目录扫描**：资源目录扫描和自定义目录扫描能发现工具，自动添加非重复工具
5. **笔记功能**：读取、保存、删除笔记正常，工具重命名时笔记正确迁移
6. **搜索标签**：工具搜索（名称/描述/路径）和标签搜索（`标签:` 前缀）功能正常
7. **Java 配置**：Java 8/11/17 路径配置保存和读取正常
8. **路径清理**：启动时自动修复配置文件和清理无效工具路径

### 技术验收

9. **配置兼容**：现有 `tool.yml` 文件可直接被 Rust 版本读写，无数据丢失
10. **事件系统**：`tool-added`、`tool-updated`、`category-deleted`、`tools-scanned` 事件正确触发和接收
11. **平台覆盖**：macOS（含 AppleScript 终端启动）完整可用；Windows（含 `CREATE_NO_WINDOW`）完整可用；Linux 基本可用
12. **二进制体积**：最终产物 ≤ 8 MB（含前端 dist）
13. **构建**：`cargo tauri build` 一键产出平台安装包（macOS `.app`/`.dmg`、Windows `.exe`/`.msi`、Linux `.deb`/`.AppImage`）

---

## 附录：Go → Rust 概念映射速查

| Go (Wails) | Rust (Tauri) | 说明 |
|---|---|---|
| `application.Service` | `#[tauri::command]` + `State<T>` | RPC + 状态管理 |
| `application.NewService(app)` | `.manage(AppState::new())` | 注册状态 |
| `ServiceStartup(ctx, options)` | `.setup(\|app\| { ... })` | 生命周期钩子 |
| `App` struct 方法 | `#[tauri::command]` 函数 | RPC 接口 |
| 自动生成 JS bindings | `@tauri-apps/api/core` `invoke()` | 前端调用 |
| `application.Get().Event.Emit` | `app_handle.emit()` | 后端→前端事件 |
| `@wailsio/runtime` `Events.On` | `@tauri-apps/api/event` `listen()` | 前端事件监听 |
| `os/exec.Command` | `std::process::Command` | 进程执行 |
| `setHideWindow(cmd)` | `Command::creation_flags(CREATE_NO_WINDOW)` | Windows 隐藏窗口 |
| `ioutil.ReadFile` | `std::fs::read_to_string` | 文件读取 |
| `ioutil.WriteFile` | `std::fs::write` | 文件写入 |
| `yaml.Marshal/Unmarshal` | `serde_yaml::to_string`/`from_str` | YAML 序列化 |
| `encoding/json` | `serde_json` | JSON 序列化 |
| `time.Time` | `chrono::DateTime<Utc>` | 时间类型 |
| `runtime.GOOS` | `std::env::consts::OS` / `#[cfg(target_os)]` | 平台检测 |
| `filepath.Join` | `std::path::PathBuf::join` | 路径拼接 |
| `os.Stat` | `std::fs::metadata` | 文件信息 |
| `os.MkdirAll` | `std::fs::create_dir_all` | 递归创建目录 |
| `filepath.Walk` | `walkdir::WalkDir` 或手动递归 | 目录遍历 |
| `//go:embed` | `include_dir!` 或 Tauri `resources` | 资源嵌入 |
| `context.Context` | 无直接等价 / `CancellationToken` | 上下文取消 |
| goroutine | `tokio::spawn` / `tauri::async_runtime::spawn` | 异步任务 |
| `strings.Fields` | `shell_words::split` | 字符串分词 |
