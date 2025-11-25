use eframe::egui;
use std::path::PathBuf;
use dirs;
use super::file_operations::{FileOperations, FileOperationResult};
use super::create_operations::generate_default_folder_name;
use super::help::HelpSystem;

pub fn show_menu_bar(
    ui: &mut egui::Ui,
    current_path: &mut PathBuf,
    show_hidden: &mut bool,
    file_operations: &mut FileOperations,
    selected_file: &Option<PathBuf>,
    help_system: &mut HelpSystem,
    view_mode: &mut super::file_list::ViewMode,
    show_drive_capacity: &mut bool,
    show_capacity_size: &mut bool,
) -> (bool, bool, bool, bool, bool) {
    let mut needs_refresh = false;
    let mut should_paste = false;
    let mut should_rename = false;
    let mut should_delete = false;
    let mut should_create_folder = false;

    egui::menu::bar(ui, |ui| {
        ui.menu_button("文件", |ui| {
            if ui.button("新建文件夹").clicked() {
                should_create_folder = true;
                ui.close_menu();
            }
            if ui.button("刷新").clicked() {
                // TODO: 实现刷新功能
                ui.close_menu();
            }
            ui.separator();
            if ui.button("退出").clicked() {
                std::process::exit(0);
            }
        });

        ui.menu_button("编辑", |ui| {
            // 复制按钮
            if let Some(ref path) = selected_file {
                if ui.button("复制").clicked() {
                    file_operations.copy_to_clipboard(vec![path.clone()]);
                    ui.close_menu();
                }

                // 重命名按钮
                if ui.button("重命名").clicked() {
                    should_rename = true;
                    ui.close_menu();
                }

                // 删除按钮
                if ui.button("删除").clicked() {
                    if let Some(ref path) = selected_file {
                        match file_operations.delete_files(&[path.clone()]) {
                            FileOperationResult::NeedsConfirmation(_) => {
                                should_delete = true;
                            }
                            FileOperationResult::Error(msg) => {
                                eprintln!("删除错误: {}", msg);
                            }
                            FileOperationResult::Success => {
                                // 这个情况不应该发生，删除总是需要确认
                            }
                        }
                    }
                    ui.close_menu();
                }
            } else {
                // 没有选中文件时禁用相关按钮
                ui.add_enabled(false, egui::Button::new("复制"));
                ui.add_enabled(false, egui::Button::new("重命名"));
                ui.add_enabled(false, egui::Button::new("删除"));
            }

            // 粘贴按钮（只要剪贴板有内容就可用）
            // 注意：这里简化处理，假设有剪贴板内容时就可用
            // 在实际使用中，你可能需要调用 file_operations.has_clipboard_content()
            if ui.button("粘贴").clicked() {
                // 粘贴功能需要在主程序中处理，因为需要知道当前路径
                should_paste = true;
                ui.close_menu();
            }

            ui.separator();
            if ui.button("全选").clicked() {
                // TODO: 实现全选功能
                ui.close_menu();
            }
        });

        ui.menu_button("查看", |ui| {
            if ui.checkbox(show_hidden, "显示隐藏文件").changed() {
                needs_refresh = true;
                ui.close_menu();
            }
            ui.separator();
            if ui.button("详细信息").clicked() {
                *view_mode = super::file_list::ViewMode::Details;
                ui.close_menu();
            }
            if ui.button("大图标").clicked() {
                *view_mode = super::file_list::ViewMode::LargeIcons;
                ui.close_menu();
            }
            if ui.button("小图标").clicked() {
                *view_mode = super::file_list::ViewMode::SmallIcons;
                ui.close_menu();
            }
            ui.separator();
            if ui.checkbox(show_drive_capacity, "硬盘容量").changed() {
                ui.close_menu();
            }
            if ui.checkbox(show_capacity_size, "容量大小").changed() {
                ui.close_menu();
            }
        });

        ui.menu_button("转到", |ui| {
            if ui.button("主页").clicked() {
                if let Some(home_dir) = dirs::home_dir() {
                    *current_path = home_dir;
                    needs_refresh = true;
                }
                ui.close_menu();
            }
            if ui.button("桌面").clicked() {
                if let Some(desktop_dir) = dirs::desktop_dir() {
                    *current_path = desktop_dir;
                    needs_refresh = true;
                }
                ui.close_menu();
            }
            if ui.button("文档").clicked() {
                if let Some(doc_dir) = dirs::document_dir() {
                    *current_path = doc_dir;
                    needs_refresh = true;
                }
                ui.close_menu();
            }
            if ui.button("下载").clicked() {
                if let Some(download_dir) = dirs::download_dir() {
                    *current_path = download_dir;
                    needs_refresh = true;
                }
                ui.close_menu();
            }
            ui.separator();
            if ui.button("上一级").clicked() {
                if let Some(parent) = current_path.parent() {
                    *current_path = parent.to_path_buf();
                    needs_refresh = true;
                }
                ui.close_menu();
            }
        });

        ui.menu_button("帮助", |ui| {
            if ui.button("关于").clicked() {
                help_system.show_about();
                ui.close_menu();
            }
        });
    });

    (needs_refresh, should_paste, should_rename, should_delete, should_create_folder)
}