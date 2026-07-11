package main

import (
	"embed"
	"log"
	"runtime"

	"github.com/wailsapp/wails/v3/pkg/application"
)

//go:embed all:frontend/dist
//go:embed build/appicon.png
var assets embed.FS

func main() {
	// 创建一个新的应用实例
	app := NewApp()

	// 创建 Wails v3 应用
	wailsApp := application.New(application.Options{
		Name: "SpearX",
		Services: []application.Service{
			application.NewService(app),
		},
		Assets: application.AssetOptions{
			Handler: application.AssetFileServerFS(assets),
		},
	})

	// 平台分离：macOS 用原生毛玻璃，Windows 用 CSS 模拟
	windowOpts := application.WebviewWindowOptions{
		Title:  "SpearX",
		Width:  1024,
		Height: 768,
	}

	if runtime.GOOS == "darwin" {
		// macOS: 原生 NSVisualEffectView 模糊背景
		windowOpts.BackgroundType = application.BackgroundTypeTranslucent
		windowOpts.Mac = application.MacWindow{
			Appearance: application.NSAppearanceNameDarkAqua,
			Backdrop:   application.MacBackdropTranslucent,
			TitleBar: application.MacTitleBar{
				AppearsTransparent: true,
				HideTitle:          true,
				FullSizeContent:    true,
			},
		}
	} else {
		// Windows/Linux: 纯色背景 + CSS 毛玻璃模拟
		windowOpts.BackgroundType = application.BackgroundTypeSolid
		windowOpts.BackgroundColour = application.NewRGBA(18, 18, 18, 255)
	}

	wailsApp.Window.NewWithOptions(windowOpts)

	if err := wailsApp.Run(); err != nil {
		log.Fatal(err)
	}
}
