package main

import (
	"embed"
	"log"

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

	// 创建主窗口（macOS 透明窗口 + 隐藏标题栏，消除 Tahoe 弃用警告）
	wailsApp.Window.NewWithOptions(application.WebviewWindowOptions{
		Title:          "SpearX",
		Width:          1024,
		Height:         768,
		BackgroundType: application.BackgroundTypeTranslucent,
		Mac: application.MacWindow{
			Appearance: application.NSAppearanceNameDarkAqua,
			Backdrop:   application.MacBackdropTranslucent,
			TitleBar: application.MacTitleBar{
				AppearsTransparent: true,
				HideTitle:          true,
				FullSizeContent:    true,
			},
		},
	})

	if err := wailsApp.Run(); err != nil {
		log.Fatal(err)
	}
}
