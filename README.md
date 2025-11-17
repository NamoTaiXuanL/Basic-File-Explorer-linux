# 文件浏览器 (File Explorer)

一个用Rust和egui开发的现代化文件浏览器，支持多种文件类型图标显示和多视图模式。

## 功能特性

### 🎨 多视图模式
- **详细信息模式**：传统的列表视图，显示文件名、大小、修改时间、类型等信息
- **大图标模式**：网格布局，显示大尺寸图标和文件名
- **小图标模式**：紧凑的网格布局，显示小尺寸图标和文件名

### 🎯 智能文件图标系统
- **文件夹图标**：自定义PNG图标（32px/64px）
- **可执行文件图标**：EXE文件专用图标（25px/50px）
- **动态链接库图标**：DLL文件专用图标（25px/50px）
- **文本文件图标**：TXT文件专用图标（25px/50px）
- **代码文件图标**：支持60+种编程语言的通用图标（25px/50px）
  - 支持语言：rs, py, js, ts, jsx, tsx, java, c, cpp, cc, cxx, h, hpp, hxx, cs, vb, php, rb, go, swift, kt, scala, clj, hs, ml, fs等
  - 脚本语言：sh, bat, cmd, ps1, pl, lua, awk, sed, vim, fish, zsh, bash, ksh, tcsh, csh
  - Web技术：html, htm, css, scss, sass, less, json, xml
  - 配置文件：yaml, yml, toml, ini, cfg, conf, log
  - 文档格式：md, markdown, r
- **无格式文件图标**：无后缀名文件专用图标（25px/50px）
- **默认文件图标**：其他未定义文件类型的通用图标（25px/50px）

### 🔧 核心功能
- **文件导航**：支持双击进入目录，单击选择文件
- **文件操作**：复制、粘贴、删除、重命名等基础文件操作
- **新建文件夹**：支持创建新文件夹
- **菜单栏功能**：
  - 文件：刷新、退出
  - 编辑：复制、粘贴
  - 查看：详细信息、大图标、小图标视图切换
  - 转到：桌面、文档、下载、音乐、图片等快速导航
  - 帮助：关于对话框
- **隐藏文件显示**：可选择是否显示系统隐藏文件

### 🖼️ 用户体验
- **中文支持**：完整的中文界面和文件名显示
- **智能对齐**：图标和文字完美对齐，视觉效果专业
- **自动刷新**：导航操作自动刷新内容
- **UTF-8安全**：正确处理中文字符，避免截断错误

### 🎨 应用程序品牌
- **自定义应用图标**：专业的软件品牌标识
- **多尺寸支持**：图标在标题栏、任务栏、Alt+Tab界面完美显示

## 技术架构

### 开发语言
- **Rust**：系统编程语言，内存安全，高性能
- **egui**：现代化的即时模式GUI库
- **image**：图像处理库，支持PNG、ICO等格式

### 模块化设计
- **app_icon**：应用程序图标管理
- **icon_manager**：文件图标管理和加载
- **file_list**：文件列表显示和交互
- **menu_bar**：菜单栏功能
- **toolbar**：工具栏功能
- **file_operations**：文件操作处理
- **preview**：文件预览功能
- **help**：帮助系统

### 项目结构
```
src/
├── main.rs                 # 主程序入口
├── components/             # 功能模块
│   ├── app_icon.rs         # 应用程序图标
│   ├── icon_manager.rs     # 文件图标管理
│   ├── file_list.rs        # 文件列表显示
│   ├── menu_bar.rs         # 菜单栏
│   ├── toolbar.rs          # 工具栏
│   ├── file_operations.rs  # 文件操作
│   ├── preview.rs          # 文件预览
│   ├── help.rs             # 帮助系统
│   └── mod.rs              # 模块声明
├── utils.rs                # 工具函数
└── Cargo.toml              # 项目配置
```

## 构建和运行

### 开发环境
1. 安装Rust工具链：https://rustup.rs/
2. 克隆项目：
   ```bash
   git clone <repository-url>
   cd "File Explorer"
   ```

### 开发构建
```bash
cargo build
cargo run
```

### 生产构建
```bash
cargo build --release
```

### 依赖项
- eframe = "0.29"
- egui = "0.29"
- serde = { version = "1.0", features = ["derive"] }
- dirs = "5.0"
- chrono = "0.4"
- image = { version = "0.24", default-features = false, features = ["png", "jpeg", "ico"] }
- winapi = { version = "0.3", features = ["shellapi", "shlobj"] }

## 图标资源

项目使用自定义PNG图标文件，位于`material/png/`目录：
- 文件夹图标：Folder_icon_02_32.png、Folder_icon_02_64.png
- 可执行文件图标：Exe_icon_0_25.png、Exe_icon_0_50.png
- DLL图标：Dll_icon_0_25.png、Dll_icon_0_50.png
- 文本文件图标：Txt_icon_0_25.png、Txt_icon_0_50.png
- 代码文件图标：Code_icon_0_25.png、Code_icon_0_50.png
- 无格式文件图标：Unidentified_icon_0_25.png、Unidentified_icon_0_50.png
- 默认文件图标：default_icon_0_25.png、default_icon_0_50.png
- 应用程序图标：logo_icon_1_150.png（转换为ICO格式）

## 版本历史

详细的版本更新记录请参考 [CHANGELOG.md](CHANGELOG.md)

## 许可证

本项目采用开源许可证，具体信息请参考LICENSE文件。

## 贡献

欢迎提交Issue和Pull Request来改进这个项目。

