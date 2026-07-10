//go:build windows

package main

import (
	"os/exec"
	"syscall"
)

// setHideWindow 在 Windows 平台隐藏弹出的控制台窗口。
// CREATE_NO_WINDOW (0x08000000) 阻止子进程创建新的控制台窗口。
func setHideWindow(cmd *exec.Cmd) {
	cmd.SysProcAttr = &syscall.SysProcAttr{
		HideWindow:    true,
		CreationFlags: 0x08000000,
	}
}
