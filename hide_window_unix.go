//go:build !windows

package main

import "os/exec"

// setHideWindow 在非 Windows 平台隐藏命令窗口。
// macOS/Linux 的 GUI 应用中 exec.Command 不会弹出终端窗口，无需特殊处理。
func setHideWindow(cmd *exec.Cmd) {
}
