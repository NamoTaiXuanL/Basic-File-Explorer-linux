use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use eframe::egui;

// 新建操作管理器
pub struct CreateOperations {
    last_error: Option<String>,
}

#[derive(Debug)]
pub enum CreateOperationResult {
    Success,
    Error(String),
    NeedsConfirmation(String), // 用于需要确认的操作（如覆盖等）
    NeedsInput(String), // 用于需要用户输入的操作（如新建文件夹名称）
}

impl CreateOperations {
    pub fn new() -> Self {
        Self {
            last_error: None,
        }
    }

    // 新建文件夹
    pub fn create_folder(&self, parent_path: &Path, folder_name: &str) -> CreateOperationResult {
        if folder_name.is_empty() {
            return CreateOperationResult::Error("文件夹名称不能为空".to_string());
        }

        // 检查文件夹名称是否包含非法字符
        if self.contains_invalid_chars(folder_name) {
            return CreateOperationResult::Error("文件夹名称包含非法字符".to_string());
        }

        let new_folder_path = parent_path.join(folder_name);

        // 检查文件夹是否已存在
        if new_folder_path.exists() {
            return CreateOperationResult::Error("文件夹已存在".to_string());
        }

        match fs::create_dir(&new_folder_path) {
            Ok(_) => CreateOperationResult::Success,
            Err(e) => CreateOperationResult::Error(format!("创建文件夹失败: {}", e)),
        }
    }

    // 生成唯一文件夹名
    pub fn generate_unique_folder_name(&self, parent_path: &Path, base_name: &str) -> String {
        let mut counter = 1;
        let mut folder_name = base_name.to_string();

        // 首先检查基本名称是否可用
        if !parent_path.join(&folder_name).exists() {
            return folder_name;
        }

        // 生成带数字的文件夹名
        loop {
            folder_name = format!("{} ({})", base_name, counter);
            let folder_path = parent_path.join(&folder_name);

            if !folder_path.exists() {
                return folder_name;
            }

            counter += 1;

            // 防止无限循环
            if counter > 9999 {
                return format!("{}_{}", base_name, chrono::Utc::now().timestamp());
            }
        }
    }

    // 显示新建文件夹对话框
    pub fn show_new_folder_dialog(&mut self, ctx: &egui::Context, default_name: &str) -> Option<String> {
        let mut folder_name = default_name.to_string();
        let mut result = None;
        let mut open = true;

        egui::Window::new("新建文件夹")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("文件夹名称:");
                    ui.text_edit_singleline(&mut folder_name);
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("确定").clicked() {
                        result = Some(folder_name);
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

    // 获取最后一个错误
    pub fn get_last_error(&self) -> Option<String> {
        self.last_error.clone()
    }

    // 私有辅助方法

    // 检查文件夹名是否包含非法字符
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

    // 验证文件夹名称
    fn validate_folder_name(&self, name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("文件夹名称不能为空".to_string());
        }

        if name.len() > 255 {
            return Err("文件夹名称过长（最多255个字符）".to_string());
        }

        if self.contains_invalid_chars(name) {
            return Err("文件夹名称包含非法字符".to_string());
        }

        // Windows 特殊名称检查
        #[cfg(target_os = "windows")]
        {
            let reserved_names = [
                "CON", "PRN", "AUX", "NUL",
                "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
                "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"
            ];

            if reserved_names.contains(&name.to_uppercase().as_str()) {
                return Err("不能使用系统保留的文件夹名称".to_string());
            }
        }

        Ok(())
    }
}

// 辅助函数：生成默认文件夹名称
pub fn generate_default_folder_name(parent_path: &Path) -> String {
    let base_name = "新建文件夹";
    let mut counter = 1;
    let mut folder_name = base_name.to_string();

    // 首先检查基本名称是否可用
    if !parent_path.join(&folder_name).exists() {
        return folder_name;
    }

    // 生成带数字的文件夹名
    loop {
        folder_name = format!("{} {}", base_name, counter);
        let folder_path = parent_path.join(&folder_name);

        if !folder_path.exists() {
            return folder_name;
        }

        counter += 1;

        // 防止无限循环
        if counter > 9999 {
            return format!("{}_{}", base_name, std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs());
        }
    }
}