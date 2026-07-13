use std::path::Path;

/// 非 macOS 平台，对齐 Go 的 "不支持的平台" 错误
pub fn open_terminal(_dir: &Path, _cmd: Option<&str>) -> Result<(), String> {
    Err("不支持的平台".to_string())
}
