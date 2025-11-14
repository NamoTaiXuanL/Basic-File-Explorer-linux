use eframe::egui;
use std::path::{Path, PathBuf};
use std::fs;
use crate::utils;

pub struct Preview {
    current_file: Option<PathBuf>,
    preview_content: String,
    file_info: FileInfo,
}

#[derive(Default)]
struct FileInfo {
    size: String,
    modified: String,
    file_type: String,
}

impl Preview {
    pub fn new() -> Self {
        Self {
            current_file: None,
            preview_content: String::new(),
            file_info: FileInfo::default(),
        }
    }

    pub fn clear(&mut self) {
        self.current_file = None;
        self.preview_content.clear();
        self.file_info = FileInfo::default();
    }

    pub fn load_preview(&mut self, path: PathBuf) {
        if self.current_file.as_ref() == Some(&path) {
            return;
        }

        self.current_file = Some(path.clone());
        self.preview_content.clear();

        // 获取文件信息
        if let Ok(metadata) = fs::metadata(&path) {
            self.file_info.size = utils::get_file_size_str(metadata.len());
            self.file_info.modified = utils::get_file_modified_time(&path)
                .unwrap_or_else(|| "未知时间".to_string());
        }

        self.file_info.file_type = self.get_file_type(&path);

        // 生成预览内容
        self.generate_preview(&path);
    }

    fn get_file_type(&self, path: &Path) -> String {
        if path.is_dir() {
            "文件夹".to_string()
        } else {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_uppercase())
                .unwrap_or_else(|| "文件".to_string())
        }
    }

    fn generate_preview(&mut self, path: &Path) {
        if path.is_dir() {
            self.generate_folder_preview(path);
        } else {
            match path.extension().and_then(|ext| ext.to_str()) {
                Some("txt") | Some("rs") | Some("js") | Some("py") | Some("html") |
                Some("css") | Some("json") | Some("xml") | Some("md") => {
                    self.generate_text_preview(path);
                }
                Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("bmp") => {
                    self.preview_content = "图片文件预览暂未实现".to_string();
                }
                _ => {
                    self.preview_content = "此文件类型不支持预览".to_string();
                }
            }
        }
    }

    fn generate_folder_preview(&mut self, path: &Path) {
        if let Ok(entries) = fs::read_dir(path) {
            let mut folders = Vec::new();
            let mut files = Vec::new();

            for entry in entries.flatten() {
                let entry_path = entry.path();
                let name = entry_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("未知")
                    .to_string();

                if entry_path.is_dir() {
                    folders.push(name);
                } else {
                    files.push(name);
                }
            }

            self.preview_content = format!(
                "文件夹内容 ({} 个文件夹, {} 个文件)\n\n📁 文件夹:\n{}\n\n📄 文件:\n{}",
                folders.len(),
                files.len(),
                folders.iter().take(20).map(|f| format!("  {}", f)).collect::<Vec<_>>().join("\n"),
                files.iter().take(20).map(|f| format!("  {}", f)).collect::<Vec<_>>().join("\n")
            );

            if folders.len() > 20 || files.len() > 20 {
                self.preview_content.push_str("\n\n... 还有更多项目");
            }
        } else {
            self.preview_content = "无法读取文件夹内容".to_string();
        }
    }

    fn generate_text_preview(&mut self, path: &Path) {
        if let Ok(content) = fs::read_to_string(path) {
            // 限制预览长度
            let lines: Vec<&str> = content.lines().collect();
            let preview_lines = lines.iter().take(100).collect::<Vec<_>>();

            self.preview_content = if lines.len() > 100 {
                format!(
                    "文本预览 (前100行，共{}行):\n\n{}",
                    lines.len(),
                    preview_lines.iter().map(|&&line| line).collect::<Vec<_>>().join("\n")
                )
            } else {
                format!(
                    "文本预览 ({}行):\n\n{}",
                    lines.len(),
                    preview_lines.iter().map(|&&line| line).collect::<Vec<_>>().join("\n")
                )
            };
        } else {
            self.preview_content = "无法读取文件内容".to_string();
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        if let Some(path) = &self.current_file {
            ui.vertical(|ui| {
                // 文件信息
                ui.group(|ui| {
                    ui.heading("文件信息");
                    ui.label(format!("名称: {}", path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("未知文件")));
                    ui.label(format!("类型: {}", self.file_info.file_type));
                    ui.label(format!("大小: {}", self.file_info.size));
                    ui.label(format!("修改时间: {}", self.file_info.modified));
                });

                ui.separator();

                // 预览内容
                if !self.preview_content.is_empty() {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.monospace(&self.preview_content);
                    });
                } else {
                    ui.label("无预览内容");
                }
            });
        } else {
            ui.label("选择一个文件查看预览");
        }
    }
}