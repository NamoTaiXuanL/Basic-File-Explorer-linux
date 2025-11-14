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
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(FileExplorerApp::new()))
        }),
    )
}

fn setup_custom_fonts(ctx: &egui::Context) {
    // 设置字体以支持中文显示
    let mut fonts = egui::FontDefinitions::default();

    // 尝试加载系统中文字体 - 使用更通用的方法
    if let Ok(font_data) = std::fs::read("C:/Windows/Fonts/msyh.ttc") {
        // 微软雅黑
        fonts.font_data.insert("microsoft_yahei".to_owned(), egui::FontData::from_owned(font_data));

        // 将中文字体添加到所有字体族
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "microsoft_yahei".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().insert(0, "microsoft_yahei".to_owned());
    } else if let Ok(font_data) = std::fs::read("C:/Windows/Fonts/simhei.ttf") {
        // 黑体
        fonts.font_data.insert("simhei".to_owned(), egui::FontData::from_owned(font_data));

        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "simhei".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().insert(0, "simhei".to_owned());
    } else if let Ok(font_data) = std::fs::read("C:/Windows/Fonts/simsun.ttc") {
        // 宋体
        fonts.font_data.insert("simsun".to_owned(), egui::FontData::from_owned(font_data));

        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "simsun".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().insert(0, "simsun".to_owned());
    } else {
        // 如果都找不到，尝试使用默认字体的备用方案
        eprintln!("警告: 未找到中文字体，中文可能显示为方块");
    }

    ctx.set_fonts(fonts);

    // 设置合适的字体大小
    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (egui::TextStyle::Heading, egui::FontId::new(18.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Body, egui::FontId::new(14.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Monospace, egui::FontId::new(13.0, egui::FontFamily::Monospace)),
        (egui::TextStyle::Button, egui::FontId::new(14.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Small, egui::FontId::new(12.0, egui::FontFamily::Proportional)),
    ].into();
    ctx.set_style(style);
}

struct FileExplorerApp {
    current_path: PathBuf,
    directory_current_path: PathBuf,  // 目录框的当前路径
    selected_file: Option<PathBuf>,
    file_list: FileList,
    directory_list: FileList,  // 使用FileList代替DirectoryTree
    preview: Preview,
    show_hidden: bool,
}

impl FileExplorerApp {
    fn new() -> Self {
        let current_path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let directory_current_path = current_path.parent().unwrap_or(&current_path).to_path_buf();
        let mut file_list = FileList::new();
        let mut directory_list = FileList::new();

        // 初始化文件列表
        file_list.refresh(current_path.clone(), false);
        directory_list.refresh(directory_current_path.clone(), false);

        Self {
            current_path: current_path.clone(),
            directory_current_path,
            selected_file: None,
            file_list,
            directory_list,
            preview: Preview::new(),
            show_hidden: false,
        }
    }

    fn navigate_to(&mut self, path: PathBuf) {
        if path.is_dir() {
            self.current_path = path.clone();
            self.file_list.refresh(path.clone(), self.show_hidden);
            self.selected_file = None;
            self.preview.clear();
        }
    }

    fn refresh_file_list(&mut self) {
        // 只刷新内容框
        self.file_list.refresh(self.current_path.clone(), self.show_hidden);
    }

    fn refresh_directory_list(&mut self) {
        // 只刷新目录框
        self.directory_list.refresh(self.directory_current_path.clone(), self.show_hidden);
    }

    fn navigate_directory_to(&mut self, path: PathBuf) {
        // 目录框导航，不刷新内容框
        if path.is_dir() {
            self.directory_current_path = path.clone();
            self.refresh_directory_list();
        }
    }

    fn go_up_directory(&mut self) {
        // 返回上级目录
        if let Some(parent) = self.directory_current_path.parent() {
            self.navigate_directory_to(parent.to_path_buf());
        }
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
                let mut needs_refresh = toolbar::show_toolbar(ui, &mut self.current_path);
                if needs_refresh {
                    self.refresh_file_list();
                    self.refresh_directory_list();
                }

                ui.separator();

                // 主内容区域 - 使用剩余的全部高度
                let available_height = ui.available_height() - 40.0; // 留一些边距
                ui.horizontal(|ui| {
                    // 左侧目录列表 (25%宽度) - 使用FileList
                    ui.allocate_ui_with_layout(
                        [ui.available_width() * 0.25, available_height].into(),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            ui.heading("目录");
                            ui.separator();

                            // 返回上级目录按钮
                            if ui.add_sized(
                                [ui.available_width(), ui.spacing().interact_size.y * 1.5],
                                egui::Button::new("⬆ 返回上级目录")
                            ).clicked() {
                                self.go_up_directory();
                            }

                            ui.separator();

                            // 独立的滚动区域
                            let mut temp_current_path = self.directory_current_path.clone();
                            egui::ScrollArea::vertical().id_salt("directory_scroll").show(ui, |ui| {
                                let should_navigate = self.directory_list.show(ui, &mut temp_current_path, &mut self.selected_file);
                                if should_navigate {
                                    // 目录框点击目录时：更新内容框到该目录
                                    self.current_path = temp_current_path.clone();
                                    self.refresh_file_list();

                                    // 目录框保持当前显示，不改变
                                }
                            });
                        }
                    );

                    // 中间文件列表 (45%宽度)
                    ui.allocate_ui_with_layout(
                        [ui.available_width() * 0.45, available_height].into(),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            ui.heading(format!("内容: {}", self.current_path.display()));
                            ui.separator();

                            // 独立的滚动区域
                            egui::ScrollArea::vertical().id_salt("file_scroll").show(ui, |ui| {
                                let should_navigate = self.file_list.show(ui, &mut self.current_path, &mut self.selected_file);
                                if should_navigate {
                                    // 内容框导航时，更新当前路径并刷新两个列表
                                    self.current_path = self.selected_file.as_ref().unwrap_or(&self.current_path).clone();
                                    self.refresh_file_list();

                                    // 同时更新目录框显示父目录
                                    self.directory_current_path = self.current_path.parent().unwrap_or(&self.current_path).to_path_buf();
                                    self.refresh_directory_list();
                                }
                            });
                        }
                    );

                    // 右侧预览面板 (30%宽度)
                    ui.allocate_ui_with_layout(
                        [ui.available_width(), available_height].into(),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            ui.heading("预览");
                            ui.separator();
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                if let Some(selected_file) = &self.selected_file {
                                    self.preview.load_preview(selected_file.clone());
                                }
                                self.preview.show(ui);
                            });
                        }
                    );
                });
            });
        });
    }
}