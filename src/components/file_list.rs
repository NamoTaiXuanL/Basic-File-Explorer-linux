use eframe::egui;
use std::path::{Path, PathBuf};
use std::fs;
use crate::utils;
use super::mouse_strategy::MouseDoubleClickStrategy;

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
    mouse_strategy: MouseDoubleClickStrategy,
    icon_manager: super::icon_manager::IconManager,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Details,    // 详细信息（列表视图）
    LargeIcons, // 大图标
    SmallIcons, // 小图标
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
            mouse_strategy: MouseDoubleClickStrategy::new(),
            icon_manager: super::icon_manager::IconManager::new(),
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
                if !show_hidden && self.is_hidden_file(&entry_path, &name) {
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

        // 确保图标已加载
        if !self.icon_manager.is_loaded() {
            let _ = self.icon_manager.load_icons();
        }
    }

    pub fn ensure_textures(&mut self, ctx: &egui::Context) {
        self.icon_manager.ensure_textures(ctx);
    }

    pub fn load_icons(&mut self) -> Result<(), String> {
        self.icon_manager.load_icons()
    }

    pub fn get_icon_manager(&self) -> &super::icon_manager::IconManager {
        &self.icon_manager
    }

    pub fn get_icon_manager_mut(&mut self) -> &mut super::icon_manager::IconManager {
        &mut self.icon_manager
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

    pub fn show(&mut self, ui: &mut egui::Ui, current_path: &mut PathBuf, selected_file: &mut Option<PathBuf>, view_mode: ViewMode) -> bool {
        // 确保纹理已加载
        self.icon_manager.ensure_textures(ui.ctx());

        match view_mode {
            ViewMode::Details => self.show_details_view(ui, current_path, selected_file),
            ViewMode::LargeIcons => self.show_icons_view(ui, current_path, selected_file, true),
            ViewMode::SmallIcons => self.show_icons_view(ui, current_path, selected_file, false),
        }
    }

    fn show_details_view(&mut self, ui: &mut egui::Ui, current_path: &mut PathBuf, selected_file: &mut Option<PathBuf>) -> bool {
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

                // 目录使用自定义图标，EXE文件使用自定义图标，其他文件使用原有emoji
                if file.is_dir {
                    // 详细信息模式使用更小的图标 (16px)
                    self.draw_folder_icon_sized(painter, name_rect.left() + 6.0, rect.center().y, 16.0);
                    let text_x = name_rect.left() + 22.0;
                    painter.with_clip_rect(name_rect).text(egui::pos2(text_x, rect.center().y), egui::Align2::LEFT_CENTER, file.name.clone(), font_id.clone(), color);
                } else if self.is_exe_file(&file.path) {
                    // EXE文件使用自定义图标 (12px)
                    self.draw_exe_icon_sized(painter, name_rect.left() + 6.0, rect.center().y, 12.0);
                    let text_x = name_rect.left() + 20.0;
                    painter.with_clip_rect(name_rect).text(egui::pos2(text_x, rect.center().y), egui::Align2::LEFT_CENTER, file.name.clone(), font_id.clone(), color);
                } else {
                    let name_text = format!("{} {}", utils::get_file_icon(&file.path), file.name);
                    painter.with_clip_rect(name_rect).text(egui::pos2(name_rect.left() + 6.0, rect.center().y), egui::Align2::LEFT_CENTER, name_text, font_id.clone(), color);
                }
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
                    *current_path = file.path.clone();
                    *selected_file = None;
                    should_navigate = true;
                } else if button_response.double_clicked() && !file.is_dir {
                    self.mouse_strategy.handle_double_click(file.path.clone());
                } else if button_response.clicked() {
                    *selected_file = Some(file.path.clone());
                }
            }
        });

        should_navigate
    }

    fn show_icons_view(&mut self, ui: &mut egui::Ui, current_path: &mut PathBuf, selected_file: &mut Option<PathBuf>, is_large: bool) -> bool {
        let mut should_navigate = false;

        egui::ScrollArea::vertical().show(ui, |ui| {
            let available_width = ui.available_width();

            // 根据大图标还是小图标设置参数
            let (icon_size, item_size, columns) = if is_large {
                (32.0, 80.0, (available_width / 100.0).max(1.0) as usize)
            } else {
                (16.0, 50.0, (available_width / 60.0).max(1.0) as usize)
            };

            // 网格布局
            ui.horizontal_wrapped(|ui| {
                for (i, file) in self.files.iter().enumerate() {
                    let is_selected = selected_file.as_ref().map_or(false, |p| p == &file.path);

                    ui.add_space(4.0);

                    // 创建图标和名称的容器
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(item_size, item_size),
                        egui::Sense::click()
                    );

                    // 绘制选中背景
                    if is_selected {
                        let visuals = ui.visuals();
                        ui.painter().rect_filled(rect, 4.0, visuals.widgets.inactive.bg_fill);
                        ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(1.0, visuals.widgets.active.fg_stroke.color));
                    }

                    let painter = ui.painter();
                    let center_y = rect.center().y;
                    let center_x = rect.center().x;
                    let font_id = if is_large {
                        ui.style().text_styles.get(&egui::TextStyle::Body).cloned().unwrap_or_else(|| egui::FontId::new(12.0, egui::FontFamily::Proportional))
                    } else {
                        ui.style().text_styles.get(&egui::TextStyle::Small).cloned().unwrap_or_else(|| egui::FontId::new(10.0, egui::FontFamily::Proportional))
                    };
                    let color = ui.visuals().text_color();

                    // 绘制图标
                    if file.is_dir {
                        // 使用自定义文件夹图标，确保图标和文字的中轴线对齐
                        if is_large {
                            // 大图标模式：使用80%大小的64px图标 (51.2px)
                            let icon_size = 64.0 * 0.8; // 51.2px
                            let icon_y = rect.top() + (item_size * 0.15) + (icon_size * 0.5);
                            self.draw_folder_icon_scaled(painter, center_x, icon_y, icon_size);
                        } else {
                            // 小图标模式：使用32px图标，确保对齐
                            let icon_size = 32.0;
                            let icon_y = rect.top() + (item_size * 0.15) + (icon_size * 0.5);
                            self.draw_folder_icon(painter, center_x - (icon_size * 0.5), icon_y, super::icon_manager::IconSize::Small);
                        }
                    } else if self.is_exe_file(&file.path) {
                        // 绘制EXE文件图标，与文件夹图标对齐
                        if is_large {
                            // 大图标模式：使用80%大小的50px图标 (40px)
                            let icon_size = 50.0 * 0.8; // 40px
                            let icon_y = rect.top() + (item_size * 0.15) + (icon_size * 0.5);
                            self.draw_exe_icon_scaled(painter, center_x, icon_y, icon_size);
                        } else {
                            // 小图标模式：使用25px图标
                            let icon_size = 25.0;
                            let icon_y = rect.top() + (item_size * 0.15) + (icon_size * 0.5);
                            self.draw_exe_icon_scaled(painter, center_x, icon_y, icon_size);
                        }
                    } else {
                        // 绘制其他文件图标（使用emoji），与文件夹图标对齐
                        let icon_text = utils::get_file_icon(&file.path);
                        let icon_y = rect.top() + (item_size * 0.15) + if is_large { 32.0 * 0.8 } else { 16.0 };
                        let icon_pos = egui::pos2(center_x, icon_y);
                        painter.text(icon_pos, egui::Align2::CENTER_CENTER, icon_text, font_id.clone(), color);
                    }

                    // 绘制文件名，确保与图标的中轴线对齐
                    let icon_height = if file.is_dir {
                        if is_large { 64.0 * 0.8 } else { 32.0 }
                    } else if self.is_exe_file(&file.path) {
                        if is_large { 50.0 * 0.8 } else { 25.0 }
                    } else {
                        if is_large { 32.0 * 0.8 } else { 16.0 }
                    };
                    let name_y = rect.top() + (item_size * 0.15) + icon_height + 8.0; // 图标下方8px间距
                    let name_pos = egui::pos2(center_x, name_y);

                    let display_name = if file.name.len() > 10 {
                        // 安全地截断字符串，避免在UTF-8字符中间截断
                        let mut char_count = 0;
                        let mut byte_end = 0;
                        for (i, _) in file.name.char_indices() {
                            if char_count >= 7 {
                                break;
                            }
                            char_count += 1;
                            byte_end = i;
                        }
                        format!("{}...", &file.name[..byte_end])
                    } else {
                        file.name.clone()
                    };
                    painter.text(name_pos, egui::Align2::CENTER_CENTER, display_name, font_id, color);

                    // 处理点击事件
                    if response.double_clicked() && file.is_dir {
                        *current_path = file.path.clone();
                        *selected_file = None;
                        should_navigate = true;
                    } else if response.double_clicked() && !file.is_dir {
                        self.mouse_strategy.handle_double_click(file.path.clone());
                    } else if response.clicked() {
                        *selected_file = Some(file.path.clone());
                    }

                    // 每行显示指定数量的项目后换行
                    if (i + 1) % columns == 0 {
                        ui.end_row();
                    }
                }
            });
        });

        should_navigate
    }

    // 检查文件是否为隐藏文件
    fn is_hidden_file(&self, file_path: &PathBuf, file_name: &str) -> bool {
        // Unix/Linux系统：以.开头的文件
        if file_name.starts_with('.') {
            return true;
        }

        // Windows系统：检查文件属性
        #[cfg(target_os = "windows")]
        {
            if let Ok(metadata) = std::fs::metadata(file_path) {
                use std::os::windows::fs::MetadataExt;
                const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
                if (metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN) != 0 {
                    return true;
                }
            }
        }

        false
    }

    // 专门用于目录框的方法：支持单双击分离逻辑（不包含ScrollArea）
    pub fn show_for_directory(&mut self, ui: &mut egui::Ui, current_path: &mut PathBuf, selected_file: &mut Option<PathBuf>) -> (bool, bool, bool) {
        let mut should_refresh_content = false;  // 单击目录时刷新内容框
        let mut should_navigate_directory = false;  // 双击目录时目录框导航
        let mut should_open_file = false;  // 双击文件时打开文件

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
            let painter = ui.painter();
            if file.is_dir {
                // 目录框也使用小图标 (16px)
                self.draw_folder_icon_sized(painter, rect.left() + 6.0, rect.center().y, 16.0);
                let text_x = rect.left() + 22.0;
                painter.with_clip_rect(rect).text(egui::pos2(text_x, rect.center().y), egui::Align2::LEFT_CENTER, file.name.clone(), font_id, color);
            } else if self.is_exe_file(&file.path) {
                // 目录框EXE文件使用小图标 (12px)
                self.draw_exe_icon_sized(painter, rect.left() + 6.0, rect.center().y, 12.0);
                let text_x = rect.left() + 20.0;
                painter.with_clip_rect(rect).text(egui::pos2(text_x, rect.center().y), egui::Align2::LEFT_CENTER, file.name.clone(), font_id, color);
            } else {
                painter.with_clip_rect(rect).text(rect.left_center() + egui::vec2(6.0, 0.0), egui::Align2::LEFT_CENTER, format!("{} {}", utils::get_file_icon(&file.path), file.name), font_id, color);
            }

            let button_response = response;

            // 处理点击事件 - 目录框特殊逻辑
            if button_response.double_clicked() && file.is_dir {
                // 双击目录：目录框进入该目录
                *current_path = file.path.clone();
                *selected_file = None;
                should_navigate_directory = true;
            } else if button_response.double_clicked() && !file.is_dir {
                // 双击文件：使用默认程序打开
                should_open_file = self.mouse_strategy.handle_double_click(file.path.clone());
            } else if button_response.clicked() && file.is_dir {
                // 单击目录：内容框刷新到该目录
                *selected_file = Some(file.path.clone());
                should_refresh_content = true;
            } else if button_response.clicked() {
                // 单击文件：仅选择
                *selected_file = Some(file.path.clone());
            }
        }

        (should_refresh_content, should_navigate_directory, should_open_file)
    }

    fn draw_folder_icon(&self, painter: &egui::Painter, x: f32, y: f32, size: super::icon_manager::IconSize) {
        if let Some(texture) = self.icon_manager.get_folder_texture(size) {
            let icon_size = match size {
                super::icon_manager::IconSize::Small => 32.0,
                super::icon_manager::IconSize::Large => 64.0,
            };

            let rect = egui::Rect::from_center_size(
                egui::pos2(x + icon_size * 0.5, y),
                egui::vec2(icon_size, icon_size)
            );

            painter.image(
                texture.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }
    }

    fn draw_folder_icon_sized(&self, painter: &egui::Painter, x: f32, y: f32, size: f32) {
        // 使用32px纹理，但缩放到指定大小
        if let Some(texture) = self.icon_manager.get_folder_texture(super::icon_manager::IconSize::Small) {
            let rect = egui::Rect::from_center_size(
                egui::pos2(x + size * 0.5, y),
                egui::vec2(size, size)
            );

            painter.image(
                texture.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }
    }

    fn draw_folder_icon_scaled(&self, painter: &egui::Painter, center_x: f32, center_y: f32, size: f32) {
        // 使用64px纹理，但缩放到指定大小
        if let Some(texture) = self.icon_manager.get_folder_texture(super::icon_manager::IconSize::Large) {
            let rect = egui::Rect::from_center_size(
                egui::pos2(center_x, center_y),
                egui::vec2(size, size)
            );

            painter.image(
                texture.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }
    }

    fn is_exe_file(&self, file_path: &PathBuf) -> bool {
        if let Some(extension) = file_path.extension() {
            if let Some(ext_str) = extension.to_str() {
                return ext_str.to_lowercase() == "exe";
            }
        }
        false
    }

    fn draw_exe_icon(&self, painter: &egui::Painter, center_x: f32, center_y: f32, size: super::icon_manager::IconSize) {
        if let Some(texture) = self.icon_manager.get_exe_texture(size) {
            let icon_size = match size {
                super::icon_manager::IconSize::Small => 25.0,
                super::icon_manager::IconSize::Large => 50.0,
            };

            let rect = egui::Rect::from_center_size(
                egui::pos2(center_x, center_y),
                egui::vec2(icon_size, icon_size)
            );

            painter.image(
                texture.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }
    }

    fn draw_exe_icon_sized(&self, painter: &egui::Painter, x: f32, y: f32, size: f32) {
        // 使用25px纹理，但缩放到指定大小
        if let Some(texture) = self.icon_manager.get_exe_texture(super::icon_manager::IconSize::Small) {
            let rect = egui::Rect::from_center_size(
                egui::pos2(x + size * 0.5, y),
                egui::vec2(size, size)
            );

            painter.image(
                texture.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }
    }

    fn draw_exe_icon_scaled(&self, painter: &egui::Painter, center_x: f32, center_y: f32, size: f32) {
        // 使用50px纹理，但缩放到指定大小
        if let Some(texture) = self.icon_manager.get_exe_texture(super::icon_manager::IconSize::Large) {
            let rect = egui::Rect::from_center_size(
                egui::pos2(center_x, center_y),
                egui::vec2(size, size)
            );

            painter.image(
                texture.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }
    }
}