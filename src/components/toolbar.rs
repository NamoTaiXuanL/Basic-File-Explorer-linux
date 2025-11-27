use eframe::egui;
use std::path::PathBuf;
use dirs;
use super::file_list::ViewMode;

pub fn show_toolbar(ui: &mut egui::Ui, current_path: &mut PathBuf, view_mode: &mut ViewMode) -> (bool, bool) {
    let mut needs_refresh = false;
    let mut should_create_folder = false;

    ui.horizontal(|ui| {
        // 导航按钮
        if ui.add(egui::Button::new("⬅️ 返回").small()).clicked() {
            if let Some(parent) = current_path.parent() {
                *current_path = parent.to_path_buf();
                needs_refresh = true;
            }
        }

        if ui.add(egui::Button::new("🏠 主页").small()).clicked() {
            if let Some(home_dir) = dirs::home_dir() {
                *current_path = home_dir;
                needs_refresh = true;
            }
        }

        ui.add_space(10.0);

        // 路径输入框
        ui.label("路径:");
        let mut path_text = current_path.to_string_lossy().to_string();
        let response = ui.add_sized(
            egui::vec2(400.0, 24.0),
            egui::TextEdit::singleline(&mut path_text)
        );

        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let new_path = PathBuf::from(&path_text);
            if new_path.exists() && new_path.is_dir() {
                *current_path = new_path;
                needs_refresh = true;
            }
        }

        ui.add_space(10.0);

        // 快捷访问按钮
        if ui.add(egui::Button::new("📁 新建文件夹").small()).clicked() {
            should_create_folder = true;
        }

        if ui.add(egui::Button::new("🔄 刷新").small()).clicked() {
            needs_refresh = true;
        }

        ui.add_space(10.0);

        // 视图切换按钮（与新建/刷新一致的small按钮样式与高度）
        ui.label("视图:");
        if ui.add(egui::Button::new("大图标").small()).clicked() {
            *view_mode = ViewMode::LargeIcons;
        }
        if ui.add(egui::Button::new("小图标").small()).clicked() {
            *view_mode = ViewMode::SmallIcons;
        }
        if ui.add(egui::Button::new("缩略图").small()).clicked() {
            *view_mode = ViewMode::ThumbnailIcons;
        }
        if ui.add(egui::Button::new("详情").small()).clicked() {
            *view_mode = ViewMode::Details;
        }

        // 右侧对齐剩余空间
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // 搜索框
            ui.label("搜索:");
            let mut search_text = String::new();
            ui.add_sized(
                egui::vec2(150.0, 24.0),
                egui::TextEdit::singleline(&mut search_text)
                    .hint_text("搜索文件...")
            );
        });
    });

    (needs_refresh, should_create_folder)
}