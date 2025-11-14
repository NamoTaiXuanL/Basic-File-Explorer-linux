use eframe::egui;
use std::path::PathBuf;
use dirs;

pub fn show_menu_bar(ui: &mut egui::Ui, current_path: &mut PathBuf, show_hidden: &mut bool) {
    egui::menu::bar(ui, |ui| {
        ui.menu_button("文件", |ui| {
            if ui.button("新建文件夹").clicked() {
                // TODO: 实现新建文件夹功能
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
            if ui.button("复制").clicked() {
                // TODO: 实现复制功能
                ui.close_menu();
            }
            if ui.button("粘贴").clicked() {
                // TODO: 实现粘贴功能
                ui.close_menu();
            }
            if ui.button("删除").clicked() {
                // TODO: 实现删除功能
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
                ui.close_menu();
            }
            ui.separator();
            if ui.button("详细信息").clicked() {
                // TODO: 切换到详细信息视图
                ui.close_menu();
            }
            if ui.button("大图标").clicked() {
                // TODO: 切换到大图标视图
                ui.close_menu();
            }
            if ui.button("小图标").clicked() {
                // TODO: 切换到小图标视图
                ui.close_menu();
            }
        });

        ui.menu_button("转到", |ui| {
            if ui.button("主页").clicked() {
                if let Some(home_dir) = dirs::home_dir() {
                    *current_path = home_dir;
                }
                ui.close_menu();
            }
            if ui.button("桌面").clicked() {
                if let Some(desktop_dir) = dirs::desktop_dir() {
                    *current_path = desktop_dir;
                }
                ui.close_menu();
            }
            if ui.button("文档").clicked() {
                if let Some(doc_dir) = dirs::document_dir() {
                    *current_path = doc_dir;
                }
                ui.close_menu();
            }
            if ui.button("下载").clicked() {
                if let Some(download_dir) = dirs::download_dir() {
                    *current_path = download_dir;
                }
                ui.close_menu();
            }
            ui.separator();
            if ui.button("上一级").clicked() {
                if let Some(parent) = current_path.parent() {
                    *current_path = parent.to_path_buf();
                }
                ui.close_menu();
            }
        });

        ui.menu_button("帮助", |ui| {
            if ui.button("关于").clicked() {
                // TODO: 显示关于对话框
                ui.close_menu();
            }
        });
    });
}