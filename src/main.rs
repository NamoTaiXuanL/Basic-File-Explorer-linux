use eframe::egui;
use std::path::{Path, PathBuf};
use std::fs;

mod components;
use components::*;

mod utils;
use utils::*;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "文件浏览器",
        options,
        Box::new(|_cc| Ok(Box::new(FileExplorerApp::new()))),
    )
}

struct FileExplorerApp {
    current_path: PathBuf,
    selected_file: Option<PathBuf>,
    directory_tree: DirectoryTree,
    file_list: FileList,
    preview: Preview,
    show_hidden: bool,
}

impl FileExplorerApp {
    fn new() -> Self {
        let current_path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));

        Self {
            current_path: current_path.clone(),
            selected_file: None,
            directory_tree: DirectoryTree::new(current_path.clone()),
            file_list: FileList::new(),
            preview: Preview::new(),
            show_hidden: false,
        }
    }

    fn navigate_to(&mut self, path: PathBuf) {
        if path.is_dir() {
            self.current_path = path.clone();
            self.directory_tree.refresh(path.clone());
            self.file_list.refresh(path.clone(), self.show_hidden);
            self.selected_file = None;
            self.preview.clear();
        }
    }

    fn refresh_current_directory(&mut self) {
        self.directory_tree.refresh(self.current_path.clone());
        self.file_list.refresh(self.current_path.clone(), self.show_hidden);
    }

    fn select_file(&mut self, file: PathBuf) {
        self.selected_file = Some(file.clone());
        self.preview.load_preview(file);
    }
}

impl eframe::App for FileExplorerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Win11风格设置
        ctx.style_mut(|style| {
            style.visuals.window_rounding = 8.0.into();
            style.visuals.window_shadow = eframe::epaint::Shadow {
                offset: egui::vec2(0.0, 4.0),
                blur: 16.0,
                spread: 0.0,
                color: egui::Color32::from_black_alpha(25),
            };
            style.spacing.item_spacing = egui::vec2(8.0, 8.0);
            style.spacing.button_padding = egui::vec2(16.0, 8.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // 顶部菜单栏和工具栏
            ui.vertical(|ui| {
                // 菜单栏
                menu_bar::show_menu_bar(ui, &mut self.current_path, &mut self.show_hidden);

                ui.separator();

                // 工具栏
                toolbar::show_toolbar(ui, &mut self.current_path);

                ui.separator();

                // 主内容区域
                ui.horizontal(|ui| {
                    // 左侧目录树 (25%宽度)
                    ui.allocate_ui_with_layout(
                        [ui.available_width() * 0.25, ui.available_height()].into(),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            ui.heading("文件夹");
                            ui.separator();
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                let mut new_path = None;
                                self.directory_tree.show(ui, &mut new_path, &mut self.selected_file);
                                if let Some(path) = new_path {
                                    self.navigate_to(path);
                                }
                            });
                        }
                    );

                    // 中间文件列表 (50%宽度)
                    ui.allocate_ui_with_layout(
                        [ui.available_width() * 0.5, ui.available_height()].into(),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            ui.heading(format!("{}", self.current_path.display()));
                            ui.separator();
                            egui::ScrollArea::both().show(ui, |ui| {
                                self.file_list.show(ui, &mut self.current_path, &mut self.selected_file);
                            });
                        }
                    );

                    // 右侧预览面板 (25%宽度)
                    ui.vertical(|ui| {
                        ui.heading("预览");
                        ui.separator();
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            if let Some(selected_file) = &self.selected_file {
                                self.preview.load_preview(selected_file.clone());
                            }
                            self.preview.show(ui);
                        });
                    });
                });
            });
        });
    }
}