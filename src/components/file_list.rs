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

        // 文件列表
        egui::ScrollArea::vertical().show(ui, |ui| {
            for file in &self.files {
                let is_selected = selected_file.as_ref().map_or(false, |p| p == &file.path);

                // 整行按钮，增大点击区域
                let button_response = ui.add_sized(
                    [ui.available_width(), ui.spacing().interact_size.y * 1.5],
                    egui::Button::new({
                        let mut text = format!("{} {}", utils::get_file_icon(&file.path), file.name);

                        // 添加文件大小信息
                        if !file.is_dir {
                            text.push_str(&format!(" {}", utils::get_file_size_str(file.size)));
                        } else {
                            text.push_str(" —");
                        }

                        // 添加修改时间
                        text.push_str(&format!(" {}", file.modified));

                        text
                    })
                    .fill(if is_selected { ui.visuals().widgets.inactive.bg_fill } else { egui::Color32::TRANSPARENT })
                    .stroke(if is_selected {
                        egui::Stroke::new(1.0, ui.visuals().widgets.active.fg_stroke.color)
                    } else {
                        egui::Stroke::NONE
                    })
                );

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

    // 专门用于目录框的方法：支持单双击分离逻辑
    pub fn show_for_directory(&mut self, ui: &mut egui::Ui, current_path: &mut PathBuf, selected_file: &mut Option<PathBuf>) -> (bool, bool) {
        let mut should_refresh_content = false;  // 单击目录时刷新内容框
        let mut should_navigate_directory = false;  // 双击目录时目录框导航

        // 文件列表
        egui::ScrollArea::vertical().show(ui, |ui| {
            for file in &self.files {
                let is_selected = selected_file.as_ref().map_or(false, |p| p == &file.path);

                // 整行按钮，增大点击区域
                let button_response = ui.add_sized(
                    [ui.available_width(), ui.spacing().interact_size.y * 1.5],
                    egui::Button::new({
                        let mut text = format!("{} {}", utils::get_file_icon(&file.path), file.name);

                        // 添加文件大小信息
                        if !file.is_dir {
                            text.push_str(&format!(" {}", utils::get_file_size_str(file.size)));
                        } else {
                            text.push_str(" —");
                        }

                        // 添加修改时间
                        text.push_str(&format!(" {}", file.modified));

                        text
                    })
                    .fill(if is_selected { ui.visuals().widgets.inactive.bg_fill } else { egui::Color32::TRANSPARENT })
                    .stroke(if is_selected {
                        egui::Stroke::new(1.0, ui.visuals().widgets.active.fg_stroke.color)
                    } else {
                        egui::Stroke::NONE
                    })
                );

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
        });

        (should_refresh_content, should_navigate_directory)
    }
}