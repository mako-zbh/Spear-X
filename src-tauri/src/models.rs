use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================
// JavaConfig — 前端期望 PascalCase 键 {Java8, Java11, Java17}
// YAML 中也是 PascalCase
// ============================================================
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct JavaConfig {
    #[serde(default)]
    pub java8: String,
    #[serde(default)]
    pub java11: String,
    #[serde(default)]
    pub java17: String,
}

// ============================================================
// YAML 专用结构体（精确匹配 tool.yml 键名）
// ============================================================
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigYaml {
    #[serde(rename = "javapath", default)]
    pub java_paths: JavaConfig,
    #[serde(rename = "Categories", default)]
    pub categories: Vec<CategoryYaml>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CategoryYaml {
    #[serde(rename = "CategoryName")]
    pub name: String,
    #[serde(rename = "Icon", default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(rename = "Tools", default)]
    pub tools: Vec<ToolYaml>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolYaml {
    #[serde(rename = "ID", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "ToolName")]
    pub name: String,
    #[serde(rename = "PATH")]
    pub path: String,
    #[serde(rename = "FileName", default)]
    pub file_name: String,
    #[serde(rename = "VALUE", default)]
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

// ============================================================
// JSON 专用结构体（Tauri IPC，camelCase）
// ============================================================
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub file_name: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub optional: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_path: Option<String>,
    #[serde(default)]
    pub open_count: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default)]
    pub tools: Vec<Tool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Categories {
    #[serde(rename = "categories", alias = "Categories", default)]
    pub categories: Vec<Category>,
}

// ============================================================
// 其他结构体
// ============================================================
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScannedTool {
    pub path: String,
    pub category: String,
    #[serde(default)]
    pub possible_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CleanupResult {
    #[serde(default)]
    pub invalid_tools_count: i32,
    #[serde(default)]
    pub invalid_categories_count: i32,
    #[serde(default)]
    pub cleaned_notes: i32,
    #[serde(default)]
    pub migrated_notes: i32,
    #[serde(default)]
    pub invalid_tool_names: Vec<String>,
    #[serde(default)]
    pub migrated_tool_names: Vec<String>,
}

// ============================================================
// From/Into 互转实现
// ============================================================
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

impl From<CategoryYaml> for Category {
    fn from(y: CategoryYaml) -> Self {
        Category {
            name: y.name,
            icon: y.icon,
            tools: y.tools.into_iter().map(Tool::from).collect(),
        }
    }
}

impl From<Category> for CategoryYaml {
    fn from(c: Category) -> Self {
        CategoryYaml {
            name: c.name,
            icon: c.icon,
            tools: c.tools.into_iter().map(ToolYaml::from).collect(),
        }
    }
}

/// CategoryInfo — 内部使用，无 serde 标签
#[derive(Debug, Clone, Default)]
pub struct CategoryInfo {
    pub name: String,
    pub icon: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tool_yml() {
        let yaml = r#"# Java配置
# 自定义Java路径配置，如果留空将使用系统默认Java
javapath:
  Java8: resources/java8/bin/java
  Java11: resources/java11/bin/java
  Java17: resources/java17/bin/java
Categories:
  - CategoryName: 信息收集
    Tools:
      - ToolName: WebFinder
        PATH: resources/info/webfinder
        FileName: webfinder-next.jar
        VALUE: Java8
        COMMAND: -jar
        Optional: ""
"#;
        let config: ConfigYaml = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.java_paths.java8, "resources/java8/bin/java");
        assert_eq!(config.java_paths.java11, "resources/java11/bin/java");
        assert_eq!(config.categories.len(), 1);
        assert_eq!(config.categories[0].name, "信息收集");
        assert_eq!(config.categories[0].tools.len(), 1);
        assert_eq!(config.categories[0].tools[0].name, "WebFinder");
        assert_eq!(config.categories[0].tools[0].path, "resources/info/webfinder");
        assert_eq!(config.categories[0].tools[0].file_name, "webfinder-next.jar");
        assert_eq!(config.categories[0].tools[0].value, "Java8");
        assert_eq!(config.categories[0].tools[0].command, "-jar");
    }

    #[test]
    fn test_tool_yaml_to_json() {
        let tool_yaml = ToolYaml {
            name: "TestTool".to_string(),
            path: "resources/test/tool".to_string(),
            file_name: "test.jar".to_string(),
            value: "Java8".to_string(),
            command: "-jar".to_string(),
            optional: String::new(),
            ..Default::default()
        };
        let tool: Tool = tool_yaml.into();
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("\"name\":\"TestTool\""));
        assert!(json.contains("\"fileName\":\"test.jar\""));
        assert!(json.contains("\"value\":\"Java8\""));
    }

    #[test]
    fn test_java_config_pascal_case() {
        let config = JavaConfig {
            java8: "path8".to_string(),
            java11: "path11".to_string(),
            java17: "path17".to_string(),
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"Java8\":\"path8\""));
        assert!(json.contains("\"Java11\":\"path11\""));
        assert!(json.contains("\"Java17\":\"path17\""));
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::config;

    #[test]
    fn test_get_categories_loads_real_config() {
        // This test verifies that get_categories() can parse the actual user config
        // The config file at ~/Library/Application Support/SpearX/tool.yml should exist
        let result = config::get_categories();
        assert!(result.is_ok(), "get_categories failed: {:?}", result.err());
        let categories = result.unwrap();
        println!("Loaded {} categories", categories.categories.len());
        for cat in &categories.categories {
            println!("  Category: {} ({} tools)", cat.name, cat.tools.len());
            for tool in &cat.tools {
                println!("    Tool: {} path={} file={} value={}", tool.name, tool.path, tool.file_name, tool.value);
            }
        }
        // Verify the JSON serialization produces camelCase keys
        let json = serde_json::to_string(&categories).unwrap();
        assert!(json.contains("\"categories\""), "JSON should have 'categories' key");
        assert!(json.contains("\"name\""), "JSON should have camelCase 'name' field");
        assert!(json.contains("\"fileName\""), "JSON should have camelCase 'fileName' field");
    }

    #[test]
    fn test_load_config_returns_valid_data() {
        let result = config::load_config();
        assert!(result.is_ok(), "load_config failed: {:?}", result.err());
        let (config_yaml, categories) = result.unwrap();
        
        // Verify JavaConfig is loaded
        println!("Java8: {}", config_yaml.java_paths.java8);
        println!("Java11: {}", config_yaml.java_paths.java11);
        println!("Java17: {}", config_yaml.java_paths.java17);
        
        // Verify categories match between YAML and JSON representations
        assert_eq!(config_yaml.categories.len(), categories.categories.len());
    }

    #[test]
    fn test_java_config_json_pascal_case() {
        let result = config::load_config();
        if let Ok((config_yaml, _)) = result {
            let json = serde_json::to_string(&config_yaml.java_paths).unwrap();
            println!("JavaConfig JSON: {}", json);
            // Frontend accesses config.Java8, config.Java11, config.Java17
            assert!(json.contains("Java8"), "JSON should have PascalCase 'Java8'");
            assert!(json.contains("Java11"), "JSON should have PascalCase 'Java11'");
            assert!(json.contains("Java17"), "JSON should have PascalCase 'Java17'");
        }
    }
}
