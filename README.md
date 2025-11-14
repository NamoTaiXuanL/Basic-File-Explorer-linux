# File Explorer (文件浏览器)

一个使用Rust和egui开发的Win11风格文件浏览器。

## 功能特性

- 🖼️ **Win11风格界面** - 极简设计，圆角边框，现代化UI
- 📁 **目录树** - 左侧可展开的文件夹树形结构
- 📋 **文件列表** - 中间显示当前目录的文件和文件夹
- 👁️ **预览面板** - 右侧显示选中文件的预览信息
- 🔧 **工具栏** - 快速导航和常用操作
- 📋 **菜单栏** - 完整的菜单功能

## 项目结构

```
src/
├── main.rs              # 主程序入口
├── utils.rs             # 工具函数
└── components/
    ├── mod.rs           # 模块定义
    ├── directory_tree.rs # 目录树组件
    ├── file_list.rs     # 文件列表组件
    ├── preview.rs       # 预览组件
    ├── menu_bar.rs      # 菜单栏组件
    └── toolbar.rs       # 工具栏组件
```

## 运行方法

### 前置要求
- Rust 1.70+
- Windows 10/11

### 构建和运行
```bash
# 克隆项目
git clone <repository-url>
cd File\ Explorer

# 构建发布版本
cargo build --release

# 运行应用
./target/release/file-explorer.exe
```

或者直接运行开发版本：
```bash
cargo run
```

## 开发计划

### 已实现
- [x] 基本的三栏布局（目录树、文件列表、预览）
- [x] Win11风格的UI设计
- [x] 文件浏览和导航功能
- [x] 文本文件预览
- [x] 基本的菜单和工具栏

### 待实现功能
- [ ] 文件操作（复制、粘贴、删除、重命名）
- [ ] 搜索功能
- [ ] 图片预览
- [ ] 文件排序和筛选
- [ ] 快捷键支持
- [ ] 主题切换（深色/浅色模式）
- [ ] 文件属性对话框

## 技术栈

- **egui** - 轻量级GUI框架
- **eframe** - egui的应用程序框架
- **dirs** - 获取系统目录
- **chrono** - 时间处理
- **image** - 图片处理支持

## 贡献

欢迎提交Issue和Pull Request来改进这个项目！

## 许可证

MIT License