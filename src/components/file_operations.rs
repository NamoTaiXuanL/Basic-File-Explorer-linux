use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use std::env;
use eframe::egui;

// 文件操作管理器
pub struct FileOperations {
    clipboard: Option<ClipboardData>,
    last_error: Option<String>,
}

#[derive(Clone)]
pub struct ClipboardData {
    pub operation: OperationType,
    pub source_paths: Vec<PathBuf>,
}

#[derive(Clone, Debug)]
pub enum OperationType {
    Copy,
    Cut,
}

#[derive(Debug)]
pub enum FileOperationResult {
    Success,
    Error(String),
    NeedsConfirmation(String), // 用于删除操作的确认
}

impl FileOperations {
    pub fn new() -> Self {
        Self {
            clipboard: None,
            last_error: None,
        }
    }

    // 复制文件/文件夹到剪贴板
    pub fn copy_to_clipboard(&mut self, paths: Vec<PathBuf>) {
        self.clipboard = Some(ClipboardData {
            operation: OperationType::Copy,
            source_paths: paths,
        });
        self.last_error = None;
    }

    // 剪切文件/文件夹到剪贴板
    pub fn cut_to_clipboard(&mut self, paths: Vec<PathBuf>) {
        self.clipboard = Some(ClipboardData {
            operation: OperationType::Cut,
            source_paths: paths,
        });
        self.last_error = None;
    }

    // 粘贴剪贴板内容到目标目录
    pub fn paste_from_clipboard(&mut self, target_dir: &Path) -> FileOperationResult {
        if let Some(clipboard_data) = &self.clipboard.clone() {
            match clipboard_data.operation {
                OperationType::Copy => {
                    for source_path in &clipboard_data.source_paths {
                        if let Err(e) = self.copy_recursive(source_path, target_dir) {
                            return FileOperationResult::Error(format!("复制失败: {}", e));
                        }
                    }
                    FileOperationResult::Success
                }
                OperationType::Cut => {
                    for source_path in &clipboard_data.source_paths {
                        if let Err(e) = self.move_file(source_path, target_dir) {
                            return FileOperationResult::Error(format!("移动失败: {}", e));
                        }
                    }
                    // 剪切后清空剪贴板
                    self.clipboard = None;
                    FileOperationResult::Success
                }
            }
        } else {
            FileOperationResult::Error("剪贴板为空".to_string())
        }
    }

    // 重命名文件/文件夹
    pub fn rename_file(&self, old_path: &Path, new_name: &str) -> FileOperationResult {
        if new_name.is_empty() {
            return FileOperationResult::Error("文件名不能为空".to_string());
        }

        // 检查新文件名是否包含非法字符
        if self.contains_invalid_chars(new_name) {
            return FileOperationResult::Error("文件名包含非法字符".to_string());
        }

        let new_path = old_path.parent()
            .unwrap_or(old_path)
            .join(new_name);

        // 检查目标文件是否已存在
        if new_path.exists() {
            return FileOperationResult::Error("目标文件已存在".to_string());
        }

        match fs::rename(old_path, &new_path) {
            Ok(_) => FileOperationResult::Success,
            Err(e) => FileOperationResult::Error(format!("重命名失败: {}", e)),
        }
    }

    // 删除文件/文件夹（需要确认）
    pub fn delete_files(&self, paths: &[PathBuf]) -> FileOperationResult {
        if paths.is_empty() {
            return FileOperationResult::Error("没有选择要删除的文件".to_string());
        }

        let file_names: Vec<String> = paths.iter()
            .filter_map(|p| p.file_name().and_then(|n| n.to_str()))
            .map(|s| s.to_string())
            .collect();

        let message = if paths.len() == 1 {
            format!("确定要删除 \"{}\" 吗？", file_names[0])
        } else {
            format!("确定要删除这 {} 个项目吗？", paths.len())
        };

        FileOperationResult::NeedsConfirmation(message)
    }

    // 执行实际的删除操作
    pub fn confirm_delete(&self, paths: &[PathBuf]) -> FileOperationResult {
        for path in paths {
            if let Err(e) = self.remove_recursive(path) {
                return FileOperationResult::Error(format!("删除失败: {}", e));
            }
        }
        FileOperationResult::Success
    }

    // 显示重命名对话框
    pub fn show_rename_dialog(&mut self, ctx: &egui::Context, file_path: &PathBuf) -> Option<String> {
        let mut new_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let mut result = None;
        let mut open = true;

        egui::Window::new("重命名")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("新名称:");
                    ui.text_edit_singleline(&mut new_name);
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("确定").clicked() {
                        result = Some(new_name);
                    }
                    if ui.button("取消").clicked() {
                        result = None;
                    }
                });
            });

        if !open {
            result = None;
        }

        result
    }

    // 显示删除确认对话框
    pub fn show_delete_confirmation_dialog(&mut self, ctx: &egui::Context, message: &str) -> Option<bool> {
        let mut result = None;
        let mut open = true;

        egui::Window::new("确认删除")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(message);
                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("确定").clicked() {
                        result = Some(true);
                    }
                    if ui.button("取消").clicked() {
                        result = Some(false);
                    }
                });
            });

        if !open {
            result = Some(false);
        }

        result
    }

    // 获取最后一个错误
    pub fn get_last_error(&self) -> Option<String> {
        self.last_error.clone()
    }

    // 检查剪贴板是否有内容
    pub fn has_clipboard_content(&self) -> bool {
        self.clipboard.is_some()
    }

    // 获取剪贴板内容描述
    pub fn get_clipboard_description(&self) -> Option<String> {
        if let Some(clipboard) = &self.clipboard {
            let count = clipboard.source_paths.len();
            let operation = match clipboard.operation {
                OperationType::Copy => "复制",
                OperationType::Cut => "剪切",
            };
            Some(format!("{} {} 个项目", operation, count))
        } else {
            None
        }
    }

    // 私有辅助方法

    // 递归复制文件/文件夹
    fn copy_recursive(&self, source: &Path, target_dir: &Path) -> io::Result<()> {
        let file_name = source.file_name().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "无效的源路径")
        })?;

        let target_path = target_dir.join(file_name);

        // 检查源是否存在
        if !source.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "源文件不存在"));
        }

        // 如果目标已存在，生成新的文件名
        let final_target_path = if target_path.exists() {
            self.generate_unique_name(&target_path)?
        } else {
            target_path
        };

        if source.is_dir() {
            // 创建目标目录
            fs::create_dir_all(&final_target_path)?;

            // 复制目录内容
            for entry in fs::read_dir(source)? {
                let entry = entry?;
                let child_source = entry.path();
                self.copy_recursive(&child_source, &final_target_path)?;
            }
        } else {
            // 复制文件，使用缓冲方式避免文件被占用的问题
            self.copy_file_with_buffer(source, &final_target_path)?;
        }

        Ok(())
    }

    // 带缓冲的文件复制，避免文件被占用的问题
    fn copy_file_with_buffer(&self, source: &Path, target: &Path) -> io::Result<()> {
        use std::fs::File;
        use std::io::{Read, Write, BufReader, BufWriter};

        let mut source_file = BufReader::new(File::open(source)?);
        let mut target_file = BufWriter::new(File::create(target)?);

        let mut buffer = [0; 8192];
        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            target_file.write_all(&buffer[..bytes_read])?;
        }

        target_file.flush()?;
        Ok(())
    }

    // 生成唯一的文件名
    fn generate_unique_name(&self, path: &Path) -> io::Result<PathBuf> {
        if !path.exists() {
            return Ok(path.to_path_buf());
        }

        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let file_stem = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        let extension = path.extension()
            .and_then(|s| s.to_str());

        let mut counter = 1;
        loop {
            let new_name = if let Some(ext) = extension {
                format!("{}_{}.{}", file_stem, counter, ext)
            } else {
                format!("{}_{}", file_stem, counter)
            };

            let new_path = parent.join(new_name);
            if !new_path.exists() {
                return Ok(new_path);
            }
            counter += 1;

            // 防止无限循环
            if counter > 9999 {
                return Err(io::Error::new(io::ErrorKind::Other, "无法生成唯一文件名"));
            }
        }
    }

    // 移动文件/文件夹
    fn move_file(&self, source: &Path, target_dir: &Path) -> io::Result<()> {
        let file_name = source.file_name().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "无效的源路径")
        })?;

        let target_path = target_dir.join(file_name);
        fs::rename(source, &target_path)?;
        Ok(())
    }

    // 递归删除文件/文件夹
    fn remove_recursive(&self, path: &Path) -> io::Result<()> {
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let child_path = entry.path();
                self.remove_recursive(&child_path)?;
            }
            fs::remove_dir(path)?;
        } else {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    // 检查文件名是否包含非法字符
    fn contains_invalid_chars(&self, name: &str) -> bool {
        #[cfg(target_os = "windows")]
        {
            let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
            name.chars().any(|c| invalid_chars.contains(&c)) || name.contains('/') || name.contains('\\')
        }

        #[cfg(not(target_os = "windows"))]
        {
            name.contains('/')
        }
    }
}