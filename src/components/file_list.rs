use eframe::egui;
use std::path::{Path, PathBuf};
use std::fs;
use crate::utils;

#[derive(Clone)]
struct FileItem {
    path: PathBuf,
    name: String,
    size: u64,
    modified: String,
    is_dir: bool,
}

pub struct FileList {
    files: Vec<FileItem>,
    sort_by: SortBy,
    sort_ascending: bool,
    col_name_ratio: f32,
    col_modified_ratio: f32,
    col_type_ratio: f32,
    col_size_ratio: f32,
}

#[derive(Debug, Clone, Copy)]
enum SortBy {
    Name,
    Size,
    Modified,
}

impl FileList {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            sort_by: SortBy::Name,
            sort_ascending: true,
            col_name_ratio: 0.5,
            col_modified_ratio: 0.2,
            col_type_ratio: 0.15,
            col_size_ratio: 0.15,
        }
    }

    pub fn refresh(&mut self, path: PathBuf, show_hidden: bool) {
        self.files.clear();

        if let Ok(entries) = fs::read_dir(&path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                let name = entry_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("未知文件")
                    .to_string();

                // 跳过隐藏文件
                if !show_hidden && name.starts_with('.') {
                    continue;
                }

                let (size, is_dir) = match entry.metadata() {
                    Ok(metadata) => (metadata.len(), metadata.is_dir()),
                    Err(_) => (0, false),
                };
                let modified = utils::get_file_modified_time(&entry_path)
                    .unwrap_or_else(|| "未知时间".to_string());

                self.files.push(FileItem {
                    path: entry_path,
                    name,
                    size,
                    modified,
                    is_dir,
                });
            }
        }

        self.sort_files();
    }

    fn sort_files(&mut self) {
        self.files.sort_by(|a, b| {
            let cmp = match self.sort_by {
                SortBy::Name => {
                    // 文件夹排在前面
                    match (a.is_dir, b.is_dir) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    }
                }
                SortBy::Size => {
                    // 文件夹排在前面，然后按大小排序
                    match (a.is_dir, b.is_dir) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.size.cmp(&b.size),
                    }
                }
                SortBy::Modified => a.modified.cmp(&b.modified),
            };

            if self.sort_ascending {
                cmp
            } else {
                cmp.reverse()
            }
        });
    }

    pub fn show(&mut self, ui: &mut egui::Ui, current_path: &mut PathBuf, selected_file: &mut Option<PathBuf>) -> bool {
        let mut should_navigate = false;
        
        // 列头与可调分隔线（内容框）
        {
            let total_w = ui.available_width();
            let name_w = (self.col_name_ratio * total_w).max(60.0);
            let modified_w = (self.col_modified_ratio * total_w).max(80.0);
            let type_w = (self.col_type_ratio * total_w).max(60.0);
            let size_w = (self.col_size_ratio * total_w).max(60.0);
            let sum = name_w + modified_w + type_w + size_w;
            let scale = total_w / sum;
            let name_w = name_w * scale;
            let modified_w = modified_w * scale;
            let type_w = type_w * scale;
            let size_w = size_w * scale;

            let row_h = ui.spacing().interact_size.y * 1.2;
            let (rect, _resp) = ui.allocate_exact_size(egui::vec2(total_w, row_h), egui::Sense::hover());

            let font_id = ui.style().text_styles.get(&egui::TextStyle::Body).cloned().unwrap_or_else(|| egui::FontId::default());
            let color = ui.visuals().text_color();

            let mut x = rect.left();
            let painter = ui.painter();
            let name_rect = egui::Rect::from_min_max(egui::pos2(x, rect.top()), egui::pos2(x + name_w, rect.bottom()));
            painter.with_clip_rect(name_rect).text(egui::pos2(name_rect.left() + 6.0, rect.center().y), egui::Align2::LEFT_CENTER, "名称", font_id.clone(), color);
            x += name_w;
            let modified_rect = egui::Rect::from_min_max(egui::pos2(x, rect.top()), egui::pos2(x + modified_w, rect.bottom()));
            painter.with_clip_rect(modified_rect).text(egui::pos2(modified_rect.left() + 6.0, rect.center().y), egui::Align2::LEFT_CENTER, "修改日期", font_id.clone(), color);
            x += modified_w;
            let type_rect = egui::Rect::from_min_max(egui::pos2(x, rect.top()), egui::pos2(x + type_w, rect.bottom()));
            painter.with_clip_rect(type_rect).text(egui::pos2(type_rect.left() + 6.0, rect.center().y), egui::Align2::LEFT_CENTER, "类型", font_id.clone(), color);
            x += type_w;
            let size_rect = egui::Rect::from_min_max(egui::pos2(x, rect.top()), egui::pos2(x + size_w, rect.bottom()));
            painter.with_clip_rect(size_rect).text(egui::pos2(size_rect.left() + 6.0, rect.center().y), egui::Align2::LEFT_CENTER, "大小", font_id.clone(), color);

            let sep_w = 4.0;
            let id1 = ui.make_persistent_id("col_sep_1");
            let id2 = ui.make_persistent_id("col_sep_2");
            let id3 = ui.make_persistent_id("col_sep_3");
            let sx1 = rect.left() + name_w;
            let sx2 = rect.left() + name_w + modified_w;
            let sx3 = rect.left() + name_w + modified_w + type_w;
            for (sx, id) in [(sx1, id1), (sx2, id2), (sx3, id3)] {
                let sep_rect = egui::Rect::from_min_max(
                    egui::pos2(sx - sep_w * 0.5, rect.top()),
                    egui::pos2(sx + sep_w * 0.5, rect.bottom()),
                );
                let resp = ui.interact(sep_rect, id, egui::Sense::drag());
                ui.painter().rect_filled(sep_rect, 0.0, ui.visuals().widgets.inactive.bg_fill.gamma_multiply(0.8));
                if resp.dragged() {
                    let dx = resp.drag_delta().x;
                    let delta_ratio = dx / total_w;
                    let min = 0.08;
                    if id == id1 {
                        let total = self.col_name_ratio + self.col_modified_ratio;
                        let new_left = (self.col_name_ratio + delta_ratio).clamp(min, total - min);
                        self.col_name_ratio = new_left;
                        self.col_modified_ratio = total - new_left;
                    } else if id == id2 {
                        let total = self.col_modified_ratio + self.col_type_ratio;
                        let new_left = (self.col_modified_ratio + delta_ratio).clamp(min, total - min);
                        self.col_modified_ratio = new_left;
                        self.col_type_ratio = total - new_left;
                    } else {
                        let total = self.col_type_ratio + self.col_size_ratio;
                        let new_left = (self.col_type_ratio + delta_ratio).clamp(min, total - min);
                        self.col_type_ratio = new_left;
                        self.col_size_ratio = total - new_left;
                    }
                }
            }
        }

        // 文件列表内容
        egui::ScrollArea::vertical().show(ui, |ui| {
            for file in &self.files {
                let is_selected = selected_file.as_ref().map_or(false, |p| p == &file.path);
                let total_w = ui.available_width();
                let name_w = (self.col_name_ratio * total_w).max(60.0);
                let modified_w = (self.col_modified_ratio * total_w).max(80.0);
                let type_w = (self.col_type_ratio * total_w).max(60.0);
                let size_w = (self.col_size_ratio * total_w).max(60.0);
                let sum = name_w + modified_w + type_w + size_w;
                let scale = total_w / sum;
                let name_w = name_w * scale;
                let modified_w = modified_w * scale;
                let type_w = type_w * scale;
                let size_w = size_w * scale;

                let row_size = egui::vec2(total_w, ui.spacing().interact_size.y * 1.5);
                let (rect, response) = ui.allocate_exact_size(row_size, egui::Sense::click());

                if is_selected {
                    let visuals = ui.visuals();
                    ui.painter().rect_filled(rect, 0.0, visuals.widgets.inactive.bg_fill);
                    ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0, visuals.widgets.active.fg_stroke.color));
                }

                let font_id = ui.style().text_styles.get(&egui::TextStyle::Body).cloned().unwrap_or_else(|| egui::FontId::default());
                let color = ui.visuals().text_color();
                let mut x = rect.left();
                let painter = ui.painter();
                let name_rect = egui::Rect::from_min_max(egui::pos2(x, rect.top()), egui::pos2(x + name_w, rect.bottom()));
                let name_text = format!("{} {}", utils::get_file_icon(&file.path), file.name);
                painter.with_clip_rect(name_rect).text(egui::pos2(name_rect.left() + 6.0, rect.center().y), egui::Align2::LEFT_CENTER, name_text, font_id.clone(), color);
                x += name_w;
                let modified_rect = egui::Rect::from_min_max(egui::pos2(x, rect.top()), egui::pos2(x + modified_w, rect.bottom()));
                painter.with_clip_rect(modified_rect).text(egui::pos2(modified_rect.left() + 6.0, rect.center().y), egui::Align2::LEFT_CENTER, file.modified.clone(), font_id.clone(), color);
                x += modified_w;
                let type_rect = egui::Rect::from_min_max(egui::pos2(x, rect.top()), egui::pos2(x + type_w, rect.bottom()));
                let file_type = if file.is_dir {
                    "文件夹".to_string()
                } else {
                    file.path.extension().and_then(|e| e.to_str()).map(|s| s.to_uppercase()).unwrap_or_else(|| "文件".to_string())
                };
                painter.with_clip_rect(type_rect).text(egui::pos2(type_rect.left() + 6.0, rect.center().y), egui::Align2::LEFT_CENTER, file_type, font_id.clone(), color);
                x += type_w;
                let size_rect = egui::Rect::from_min_max(egui::pos2(x, rect.top()), egui::pos2(x + size_w, rect.bottom()));
                let size_text = if file.is_dir { "—".to_string() } else { utils::get_file_size_str(file.size) };
                painter.with_clip_rect(size_rect).text(egui::pos2(size_rect.left() + 6.0, rect.center().y), egui::Align2::LEFT_CENTER, size_text, font_id.clone(), color);

                let button_response = response;

                // 处理点击事件
                if button_response.double_clicked() && file.is_dir {
                    // 双击进入文件夹
                    *current_path = file.path.clone();
                    *selected_file = None;
                    should_navigate = true;
                } else if button_response.clicked() {
                    // 单击选择文件或文件夹
                    *selected_file = Some(file.path.clone());
                }
            }
        });

        should_navigate
    }

    // 专门用于目录框的方法：支持单双击分离逻辑（不包含ScrollArea）
    pub fn show_for_directory(&mut self, ui: &mut egui::Ui, current_path: &mut PathBuf, selected_file: &mut Option<PathBuf>) -> (bool, bool) {
        let mut should_refresh_content = false;  // 单击目录时刷新内容框
        let mut should_navigate_directory = false;  // 双击目录时目录框导航

        // 文件列表 - 不包含ScrollArea，由调用者提供
        for file in &self.files {
            let is_selected = selected_file.as_ref().map_or(false, |p| p == &file.path);

            let total_w = ui.available_width();
            let row_size = egui::vec2(total_w, ui.spacing().interact_size.y * 1.5);
            let (rect, response) = ui.allocate_exact_size(row_size, egui::Sense::click());

            if is_selected {
                let visuals = ui.visuals();
                ui.painter().rect_filled(rect, 0.0, visuals.widgets.inactive.bg_fill);
                ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0, visuals.widgets.active.fg_stroke.color));
            }

            let font_id = ui.style().text_styles.get(&egui::TextStyle::Body).cloned().unwrap_or_else(|| egui::FontId::default());
            let color = ui.visuals().text_color();
            ui.painter().with_clip_rect(rect).text(rect.left_center() + egui::vec2(6.0, 0.0), egui::Align2::LEFT_CENTER, format!("{} {}", utils::get_file_icon(&file.path), file.name), font_id, color);

            let button_response = response;

            // 处理点击事件 - 目录框特殊逻辑
            if button_response.double_clicked() && file.is_dir {
                // 双击目录：目录框进入该目录
                *current_path = file.path.clone();
                *selected_file = None;
                should_navigate_directory = true;
            } else if button_response.clicked() && file.is_dir {
                // 单击目录：内容框刷新到该目录
                *selected_file = Some(file.path.clone());
                should_refresh_content = true;
            } else if button_response.clicked() {
                // 单击文件：仅选择
                *selected_file = Some(file.path.clone());
            }
        }

        (should_refresh_content, should_navigate_directory)
    }
}