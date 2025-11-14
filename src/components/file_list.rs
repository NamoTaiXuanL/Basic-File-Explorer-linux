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

        // 表头
        ui.horizontal(|ui| {
            ui.label("名称");
            ui.allocate_ui_with_layout([200.0, ui.spacing().interact_size.y].into(), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("大小");
            });
            ui.allocate_ui_with_layout([100.0, ui.spacing().interact_size.y].into(), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("修改时间");
            });
        });

        ui.separator();

        // 文件列表
        egui::ScrollArea::vertical().show(ui, |ui| {
            for file in &self.files {
                let response = ui.horizontal(|ui| {
                    // 图标
                    let icon = utils::get_file_icon(&file.path);
                    ui.label(icon);

                    // 文件名
                    let name_response = ui.selectable_label(
                        selected_file.as_ref().map_or(false, |p| p == &file.path),
                        &file.name,
                    );

                    if name_response.double_clicked() && file.is_dir {
                        // 双击进入文件夹
                        *current_path = file.path.clone();
                        *selected_file = None;
                        should_navigate = true;
                    } else if name_response.clicked() {
                        if file.is_dir {
                            // 单击目录时高亮，不导航
                            *selected_file = Some(file.path.clone());
                        } else {
                            // 选择文件
                            *selected_file = Some(file.path.clone());
                        }
                    }

                    ui.allocate_ui_with_layout([200.0, ui.spacing().interact_size.y].into(), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // 大小
                        if !file.is_dir {
                            ui.label(utils::get_file_size_str(file.size));
                        } else {
                            ui.label("—");
                        }
                    });
                    ui.allocate_ui_with_layout([100.0, ui.spacing().interact_size.y].into(), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // 修改时间
                        ui.label(&file.modified);
                    });

                    false
                }).inner;

                if response {
                    return;
                }
            }
        });

        should_navigate
    }
}