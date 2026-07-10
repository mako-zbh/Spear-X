# SpearX - 现代化跨平台工具管理器

<div align="center">

<img src="https://img.shields.io/badge/SpearX-Tool%20Manager-6366f1?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQiIGhlaWdodD0iMjQiIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHBhdGggZD0iTTEyIDJMMTMuMDkgOC4yNkwyMCA5TDEzLjA5IDE1Ljc0TDEyIDIyTDEwLjkxIDE1Ljc0TDQgOUwxMC45MSA4LjI2TDEyIDJaIiBmaWxsPSJ3aGl0ZSIvPgo8L3N2Zz4K" alt="SpearX Logo" />

[![Wails](https://img.shields.io/badge/Wails-v2.10.1-FF6B6B?style=flat&logo=data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQiIGhlaWdodD0iMjQiIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSI+CjxwYXRoIGQ9Ik0xMiAyTDEzLjA5IDguMjZMMjAgOUwxMy4wOSAxNS43NEwxMiAyMkwxMC45MSAxNS43NEw0IDlMMTAuOTEgOC4yNkwxMiAyWiIgZmlsbD0iI0ZGNkI2QiIvPgo8L3N2Zz4K)](https://wails.io/)
[![Go Version](https://img.shields.io/badge/Go-1.24+-00ADD8?style=flat&logo=go)](https://golang.org/)
[![Vue 3](https://img.shields.io/badge/Vue.js-3.0+-4FC08D?style=flat&logo=vue.js)](https://vuejs.org/)
[![License](https://img.shields.io/badge/License-MIT-green.svg?style=flat)](LICENSE)

**一款现代化的跨平台工具管理器，为开发者和安全研究人员打造**  
**集成多种工具类型，支持智能分类管理和一键执行**  
**在极简中发现极致，在安静中感受力量。**

**Created by Spe4r** ⚡

[📖 功能特性](#-功能特性) • [🚀 快速开始](#-快速开始) • [📚 使用指南](#-使用指南) • [🔨 开发构建](#-开发构建) • [🤝 贡献指南](#-贡献指南)

</div>

---

## 📋 目录

- [✨ 功能特性](#-功能特性)
- [🎯 核心优势](#-核心优势)
- [📸 截图预览](#-截图预览)
- [🚀 快速开始](#-快速开始)
- [📚 使用指南](#-使用指南)
- [⚙️ 配置说明](#️-配置说明)
- [🔨 开发构建](#-开发构建)
- [🏗️ 架构设计](#️-架构设计)
- [❓ 常见问题](#-常见问题)
- [🤝 贡献指南](#-贡献指南)

## ✨ 功能特性

### 🎯 核心功能
- **🔧 多工具类型支持**
  - ☕ **Java 应用** (Java 8/11/17) - 支持 JAR 包执行，多版本 Java 环境
  - 🖥️ **终端工具** - 在终端中打开工具目录，支持命令行工具
  - 🌐 **Web 应用/网站** - 浏览器中打开 URL
  - 📱 **本地应用程序** - 系统默认方式打开 APP、目录等
  - 🔗 **自定义命令** - 灵活配置执行参数和命令

- **📁 智能管理**
  - 📂 分类组织管理
  - 🏷️ 标签系统
  - 🔍 实时搜索过滤
  - 📝 工具笔记和文档
  - 🔄 自动扫描发现工具

- **🎨 现代化界面**
  - 🌟 毛玻璃效果 
  - 🎭 流畅动画交互
  - 📱 响应式设计
  - 🌙 优雅的视觉效果
  - ⚡ 高性能渲染

### 🎯 核心优势

| 特性            | 说明                    | 优势               |
| --------------- | ----------------------- | ------------------ |
| **多路径支持**  | 相对路径、绝对路径、URL | 灵活的工具组织方式 |
| **智能扫描**    | 自动发现工具目录        | 快速导入现有工具   |
| **Java 多版本** | 支持 Java 8/11/17       | 兼容不同版本需求   |
| **实时搜索**    | 名称、标签、描述搜索    | 快速定位目标工具   |
| **配置管理**    | YAML 配置文件           | 可视化配置管理     |

## 📸 截图预览

### 主界面展示
> 现代化的工具管理界面，支持分类查看和智能搜索

![image-20250815025017223](https://s2.loli.net/2025/08/15/NSps9d6oXIRqzxM.png)

## 🚀 快速开始

### 📋 系统要求

| 平台        | 最低版本       |
| ----------- | -------------- |
| **macOS**   | 10.15 Catalina |

### 📦 安装方式

#### 方式一：下载预编译版本 (推荐)
1. 前往 [Releases](https://github.com/sspsec/Spear/releases) 页面
2. 下载对应平台的最新版本
3. 解压并运行 `SpearX`

#### 方式二：从源码编译
```bash
# 1. 克隆仓库
git clone https://github.com/sspsec/Spear.git
cd spear-x

# 2. 安装 Wails CLI (如果未安装)
go install github.com/wailsapp/wails/v2/cmd/wails@latest

# 3. 构建应用
wails build -platform darwin/amd64 -clean

# 4. 运行应用
./build/bin/SpearX
```

### ⚡ 快速体验

1. **首次启动**：应用会自动创建默认配置
2. **添加工具**：点击 ➕ 按钮手动添加工具
3. **扫描工具**：使用 🔄 按钮扫描现有工具目录
4. **执行工具**：点击工具卡片即可执行

## 📚 使用指南

### 📖 基本操作

#### 1️⃣ 工具添加
```bash
# 三种添加方式：
1. ➕ 手动添加 - 逐个配置工具信息
2. 🔄 扫描默认目录 - 扫描 resources 目录
3. 📂 扫描自定义目录 - 选择任意目录扫描
```

#### 2️⃣ 工具执行方式详解

##### ☕ Java 应用 (Java8/Java11/Java17)

**适用场景**：Java 开发的桌面应用、安全工具、开发工具等

- **支持格式**：`.jar` 文件
- **自动配置**：根据工具需求选择对应 Java 版本
- **内存管理**：支持自定义 JVM 参数如 `-Xmx2g`
- **环境隔离**：每个工具可独立配置 Java 环境

**配置示例**：
```yaml
- ToolName: Burp Suite Professional
  PATH: /Applications/Security/BurpSuite
  FileName: burpsuite_pro.jar
  VALUE: Java11                    # 使用 Java 11
  Optional: "-Xmx4g -XX:+UseG1GC"  # JVM 优化参数
```

**执行过程**：
1. 检测配置的 Java 版本路径
2. 如果未配置则使用系统默认 Java
3. 构建执行命令：`{java_path} {optional} -jar {jar_file}`
4. 在工具目录中执行命令

---

##### 🖥️ 终端工具 (openterm)

**适用场景**：命令行工具、脚本、需要终端交互的工具
- **支持格式**：可执行文件、脚本文件、工具目录
- **自动定位**：自动 `cd` 到工具目录
- **终端打开**：使用系统默认终端应用
- **环境保持**：保持工具的工作目录环境

**配置示例**：
```yaml
- ToolName: Nmap 网络扫描
  PATH: /usr/local/bin/nmap
  FileName: ""                     # 工具目录，无需指定文件
  VALUE: openterm
  Optional: ""
```

**执行过程**：
1. 打开系统默认终端
2. 自动切换到工具目录
3. 用户可直接使用命令行操作
4. 适合需要交互式操作的工具

---

##### 🌐 Web 应用 (Browser)

**适用场景**：在线工具、Web 界面、本地 HTML 工具

- **支持格式**：URL 链接、本地 HTML 文件
- **浏览器兼容**：使用系统默认浏览器
- **本地支持**：支持 `file://` 协议的本地文件
- **参数传递**：支持 URL 参数和锚点

**配置示例**：
```yaml
# 在线工具
- ToolName: GitHub
  PATH: https://github.com
  VALUE: Browser

# 本地 HTML 工具  
- ToolName: 本地报告查看器
  PATH: /Users/tools/reports/index.html
  VALUE: Browser
```

**执行过程**：
1. 检测路径类型（URL 或本地文件）
2. 调用系统默认浏览器
3. 传递完整路径或 URL
4. 支持传递额外参数

---

##### 📱 系统默认打开 (Open)

**适用场景**：本地应用程序、文档、媒体文件
- **支持格式**：`.app`(macOS)、各种文档目录
- **系统集成**：使用操作系统的文件关联
- **权限处理**：自动处理应用权限问题

**配置示例**：
```yaml
# macOS 应用
- ToolName: Wireshark
  PATH: /Applications/Wireshark.app
  VALUE: Open


# 文档文件
- ToolName: 工具使用手册
  PATH: /Users/docs/manual.pdf
  VALUE: Open
```

**执行过程**：
1. 检测操作系统类型
2. 使用系统 API 打开文件
3. 自动调用关联的应用程序
4. 处理权限和安全提示

---

##### 🔗 自定义命令 (Custom)

**适用场景**：复杂的启动流程、需要特殊参数的工具
- **完全自定义**：可以配置任意的执行命令
- **变量支持**：支持路径、文件名等变量替换
- **环境变量**：可设置特定的环境变量
- **脚本支持**：支持执行复杂的启动脚本

**配置示例**：
```yaml
- ToolName: Docker 工具容器
  PATH: /path/to/tool
  COMMAND: "docker run -it --rm -v {path}:/workspace tool:latest"
  VALUE: Custom
  Optional: "--network host"

- ToolName: Python 脚本工具
  PATH: /tools/scanner
  FileName: scanner.py
  COMMAND: "python3 {filename} --config config.json"
  VALUE: Custom
```

**变量说明**：
```bash
{path}      # 工具目录的绝对路径
{filename}  # 指定文件的完整路径
{name}      # 工具名称
{optional}  # Optional 字段的内容
```

---

### 🔧 执行优先级和回退机制

SpearX 提供智能的执行方式检测和回退机制：

| 优先级 | 检测方式         | 回退策略                     |
| ------ | ---------------- | ---------------------------- |
| **1**  | 用户指定的 VALUE | 按配置执行                   |
| **2**  | 文件扩展名检测   | `.jar` → Java, `.app` → Open |
| **3**  | 文件内容分析     | 检测文件头、执行权限         |
| **4**  | 系统默认关联     | 使用操作系统文件关联         |

**智能检测示例**：
```yaml
# 自动检测为 Java 应用
- ToolName: 自动检测工具
  PATH: /tools/scanner
  FileName: scanner.jar
  VALUE: ""                    # 留空，系统自动检测为 Java

# 自动检测为终端工具  
- ToolName: 命令行工具
  PATH: /tools/nmap
  FileName: ""                 # 目录形式，自动检测为 openterm
  VALUE: ""
```

#### 3️⃣ 搜索技巧
```bash
# 搜索语法：
- 名称搜索：直接输入工具名称
- 标签搜索：输入 标签名
- 描述搜索：搜索工具描述内容
```

### 📂 目录结构与组织方式

#### 🗂️ 推荐的工具组织方式：
```
工具目录/
├── 📁 信息收集/
│   ├── 📁 WebFinder/
│   │   ├── 📄 webfinder-next.jar           # 主要可执行文件
│   │   ├── 📄 WebFinder.md                 # 工具说明文档 (自动识别)
│   │   ├── 📄 config.json                  # 配置文件
│   │   └── 📁 reports/                     # 输出目录
│   ├── 📁 ScanDir/
│   │   ├── 📄 scandir-3.0.jar
│   │   ├── 📄 README.txt                   # 支持多种文档格式
│   │   └── 📁 wordlists/                   # 依赖文件
│   └── 📁 Nmap/
│       ├── 📄 nmap                         # 可执行文件
│       ├── 📁 scripts/                     # NSE 脚本
│       └── 📄 使用说明.md                  # 中文文档名支持
├── 📁 漏洞扫描/
│   ├── 📁 XrayGUI/
│   │   ├── 📄 super-xray-1.7.jar
│   │   ├── 📄 config.yaml
│   │   └── 📁 pocs/                        # 漏洞检测脚本
│   └── 📁 Nuclei/
│       ├── 📄 nuclei                       # 二进制文件
│       └── 📁 templates/                   # 模板目录
├── 📁 渗透测试/
│   ├── 📁 MiTian/
│   │   ├── 📄 mitan-jar-with-dependencies.jar
│   │   └── 📄 参数说明.md
│   ├── 📁 BurpSuite/
│   │   ├── 📄 burpsuite_pro.jar
│   │   ├── 📁 extensions/                  # 插件目录
│   │   └── 📁 projects/                    # 项目文件
│   └── 📁 Metasploit/
│       ├── 📄 msfconsole                   # 脚本文件
│       └── 📁 modules/                     # 模块目录
├── 📁 Web应用/
│   ├── 📁 本地工具/
│   │   ├── 📄 index.html                   # HTML 工具
│   │   ├── 📄 style.css
│   │   └── 📄 script.js
│   └── 📁 在线服务/                        # 快捷链接集合
│       ├── 📄 GitHub.url                   # Windows 快捷方式
│       └── 📄 VirusTotal.webloc            # macOS 快捷方式
└── 📁 系统工具/
    ├── 📁 监控工具/
    │   ├── 📄 Activity Monitor.app         # macOS 应用
    │   └── 📄 Process Monitor.exe          # Windows 应用
    └── 📁 文档工具/
        ├── 📄 工具使用手册.pdf
        ├── 📄 快速参考.docx
        └── 📁 截图示例/
```

#### 🎯 智能目录扫描特性

SpearX 在扫描目录时会智能识别以下内容：

**📋 自动识别的文件类型**：
```bash
# 可执行文件
*.jar, *.exe, *.app, *.deb, *.rpm, *.dmg
*.sh, *.bat, *.cmd, *.ps1, *.py, *.rb, *.pl

# 文档文件 (自动作为工具说明)
*.md, *.txt, *.rst, *.pdf, *.doc, *.docx
README.*, 说明.*, 使用手册.*

# Web 文件
*.html, *.htm, index.*

# 配置文件
*.json, *.yaml, *.yml, *.xml, *.ini, *.conf

# 快捷方式
*.url, *.webloc, *.lnk, *.desktop
```

**🔍 智能分类规则**：
```bash
# 根据目录名自动分类
"info", "信息收集", "reconnaissance" → 信息收集
"scan", "扫描", "vulnerability"     → 漏洞扫描  
"exploit", "渗透", "penetration"   → 渗透测试
"web", "网站", "browser"           → Web 应用
"system", "系统", "monitor"        → 系统工具

# 根据文件特征自动判断执行方式
*.jar                    → Java 应用
可执行权限的文件          → 终端工具
*.app, *.exe            → 系统默认打开
http://, https://       → 浏览器打开
*.html, *.htm           → 浏览器打开
```

#### 路径类型支持：
```yaml
# 相对路径 (传统方式)
PATH: resources/info/webfinder

# 绝对路径 (新增支持)
PATH: /Users/username/tools/webfinder

# URL 路径 (Web 应用)
PATH: https://github.com
```

## ⚙️ 配置说明

### 📄 tool.yml 配置文件

```yaml
# Java 环境配置
JavaPaths:
  Java8: "/usr/bin/java"          # Java 8 路径 (可选)
  Java11: "/usr/bin/java"         # Java 11 路径 (可选)
  Java17: "/usr/bin/java"         # Java 17 路径 (可选)

# 工具分类配置
Categories:
- CategoryName: 信息收集           # 分类名称
  Tools:                         # 工具列表
  - ToolName: WebFinder         # 工具名称
    PATH: resources/info/webfinder # 工具路径
    FileName: webfinder-next.jar   # 主要文件
    VALUE: Java8                   # 执行方式
    COMMAND: -jar                  # 执行命令
    Optional: "-Xmx2g"            # 可选参数
```

### 🔧 Java 环境配置

#### 自动检测 (推荐)
- 留空 `JavaPaths` 配置，系统自动使用环境变量中的 Java

#### 手动配置
```yaml
JavaPaths:
  Java8: "/Library/Java/JavaVirtualMachines/jdk1.8.0_301.jdk/Contents/Home/bin/java"
  Java11: "/Library/Java/JavaVirtualMachines/jdk-11.0.12.jdk/Contents/Home/bin/java"
  Java17: "/Library/Java/JavaVirtualMachines/jdk-17.0.2.jdk/Contents/Home/bin/java"
```
<img width="1024" height="768" alt="图片" src="https://github.com/user-attachments/assets/50058bb9-1198-4017-a288-ed2a502884ae" />




## 🔨 开发构建

### 🛠️ 开发环境要求

```bash
# 必需工具
Go 1.24+                    # 后端开发语言
Node.js 18+                 # 前端构建工具
Wails CLI v2.10.1           # 桌面应用框架

# 安装 Wails CLI
go install github.com/wailsapp/wails/v2/cmd/wails@latest

# 验证安装
wails doctor
```

### 📦 生产构建

```bash
# 构建当前平台
wails build

# 跨平台构建
wails build -platform darwin/arm64    # macOS Apple Silicon
wails build -platform darwin/amd64    # macOS Intel
wails build -platform windows/amd64   # Windows 64位
wails build -platform linux/amd64     # Linux 64位

# 构建后清理
wails build -clean
```

### 🔧 技术栈

#### 前端技术
- **框架**: Vue 3 + Composition API
- **UI 库**: Element Plus
- **构建工具**: Vite
- **样式**: CSS3 + 毛玻璃效果
- **图标**: Element Plus Icons

#### 后端技术
- **语言**: Go 1.24
- **框架**: Wails v2
- **配置**: YAML
- **依赖**: 标准库为主

## 🏗️ 架构设计

### 📐 整体架构

```
┌─────────────────┐    ┌─────────────────┐
│   Frontend      │    │   Backend       │
│   (Vue 3)       │◄──►│   (Go)          │
├─────────────────┤    ├─────────────────┤
│ • UI 组件       │    │ • 工具管理      │
│ • 状态管理      │    │ • 文件操作      │
│ • 事件处理      │    │ • 命令执行      │
│ • 毛玻璃效果    │    │ • 配置管理      │
└─────────────────┘    └─────────────────┘
         ▲                       ▲
         │      Wails Bridge     │
         ▼                       ▼
┌─────────────────┐    ┌─────────────────┐
│   Desktop       │    │   System        │
│   Runtime       │    │   Integration   │
└─────────────────┘    └─────────────────┘
```

### 🧩 核心模块

| 模块           | 职责           | 主要功能             |
| -------------- | -------------- | -------------------- |
| **工具管理器** | 工具 CRUD 操作 | 增删改查、分类管理   |
| **执行引擎**   | 工具执行控制   | 多类型执行、参数处理 |
| **配置管理器** | 配置文件处理   | YAML 解析、配置更新  |
| **文件浏览器** | 目录和文件操作 | 扫描、浏览、选择     |
| **界面控制器** | UI 状态管理    | 组件通信、状态同步   |

## ❓ 常见问题

### 🔧 环境配置问题

**Q: 运行程序显示文件损坏不能打开？**

> **首次运行提示验证签名失败**，执行以下命令可以正常启动：
>
> ```
> xattr -rd com.apple.quarantine SpearX.app
> 或
> codesign --sign - SpearX.app
> ```
>

**Q: Java 工具无法执行？**
> A: 请检查 Java 环境配置：
> 1. 确认已安装对应 Java 版本
> 2. 在设置中配置正确的 Java 路径
> 3. 或保持路径为空使用系统默认 Java

**Q: 工具扫描没有发现工具？**
> A: 请确认：
> 1. 目录路径正确且可访问
> 2. 目录中包含可执行文件 (.jar, .exe, .app 等)
> 3. 具有目录读取权限

### 🚀 使用问题

**Q: 如何添加自定义工具？**
> A: 两种方式：
> 1. 点击 ➕ 按钮手动添加
> 2. 使用 📂 按钮扫描包含工具的目录

**Q: 工具笔记保存在哪里？**
> A: 笔记以 Markdown 格式保存在工具目录下，文件名为 `工具名.md`

**Q: 支持哪些文件格式？**
> A: 支持格式包括：
> - Java: `.jar`
> - Windows: `.exe`, `.bat`, `.cmd`
> - macOS: `.app`, `.sh`
> - Linux: `.sh`, `.py`, 可执行文件
> - Web: URL 链接

### 📂 配置问题

**Q: 如何备份工具配置？**
> A: 备份以下文件：
> - `tool.yml` - 工具配置
> - 工具目录中的 `.md` 笔记文件



