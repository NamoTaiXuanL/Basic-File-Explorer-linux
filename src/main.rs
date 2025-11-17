use eframe::egui;
use std::path::{Path, PathBuf};
use std::fs;

mod components;
use components::*;
use components::app_icon::*;

mod utils;
use utils::*;

fn main() -> Result<(), eframe::Error> {
    // 加载应用程序图标
    let icon_data = load_app_icon();

    let mut viewport_builder = egui::ViewportBuilder::default()
        .with_inner_size([1400.0, 900.0])
        .with_resizable(true);

    // 如果图标加载成功，设置窗口图标
    if let Some(icon) = icon_data {
        viewport_builder = viewport_builder.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport: viewport_builder,
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
    file_operations: FileOperations,
    create_operations: CreateOperations,
    help_system: HelpSystem,
    drive_bar: DriveBar,  // 新增盘符栏
    show_hidden: bool,
    nav_history: Vec<PathBuf>,
    history_pos: usize,
    left_ratio: f32,
    mid_ratio: f32,
    // 对话框状态
    show_rename_dialog: bool,
    rename_input: String,
    show_delete_confirmation: bool,
    delete_confirmation_message: String,
    show_new_folder_dialog: bool,
    new_folder_name: String,
    view_mode: components::file_list::ViewMode,
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

        // 加载图标
        let _ = file_list.load_icons();
        let _ = directory_list.load_icons();

        Self {
            current_path: current_path.clone(),
            directory_current_path,
            selected_file: None,
            file_list,
            directory_list,
            preview: Preview::new(),
            file_operations: FileOperations::new(),
            create_operations: CreateOperations::new(),
            help_system: HelpSystem::new(),
            drive_bar: DriveBar::new(&current_path),
            show_hidden: false,
            nav_history: vec![current_path.clone()],
            history_pos: 0,
            left_ratio: 0.25,
            mid_ratio: 0.45,
            show_rename_dialog: false,
            rename_input: String::new(),
            show_delete_confirmation: false,
            delete_confirmation_message: String::new(),
            show_new_folder_dialog: false,
            new_folder_name: String::new(),
            view_mode: components::file_list::ViewMode::Details,
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
        // 保存工作区状态
        self.save_current_workspace_state();
    }

    fn refresh_directory_list(&mut self) {
        // 只刷新目录框
        self.directory_list.refresh(self.directory_current_path.clone(), self.show_hidden);
        // 保存工作区状态
        self.save_current_workspace_state();
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

    fn push_history(&mut self, path: PathBuf) {
        if self.history_pos + 1 < self.nav_history.len() {
            self.nav_history.truncate(self.history_pos + 1);
        }
        self.nav_history.push(path.clone());
        self.history_pos = self.nav_history.len() - 1;
    }

    fn can_go_back(&self) -> bool { self.history_pos > 0 }
    fn can_go_forward(&self) -> bool { self.history_pos + 1 < self.nav_history.len() }

    fn go_back(&mut self) {
        if self.can_go_back() {
            self.history_pos -= 1;
            let path = self.nav_history[self.history_pos].clone();
            self.current_path = path;
            self.refresh_file_list();
        }
    }

    fn go_forward(&mut self) {
        if self.can_go_forward() {
            self.history_pos += 1;
            let path = self.nav_history[self.history_pos].clone();
            self.current_path = path;
            self.refresh_file_list();
        }
    }

    fn save_current_workspace_state(&mut self) {
        self.drive_bar.save_workspace_state(
            &self.current_path,
            &self.directory_current_path,
            &self.nav_history,
            self.history_pos
        );
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
                let (menu_needs_refresh, menu_should_paste, menu_should_rename, menu_should_delete, menu_should_create_folder) =
                    menu_bar::show_menu_bar(ui, &mut self.current_path, &mut self.show_hidden, &mut self.file_operations, &self.selected_file, &mut self.help_system, &mut self.view_mode);

                // 处理菜单栏的刷新请求（来自查看和转到功能）
                if menu_needs_refresh {
                    self.refresh_file_list();
                    self.refresh_directory_list();
                }

                // 处理菜单栏的粘贴请求
                if menu_should_paste {
                    match self.file_operations.paste_from_clipboard(&self.current_path) {
                        FileOperationResult::Success => {
                            self.refresh_file_list();
                            self.refresh_directory_list();
                        }
                        FileOperationResult::Error(msg) => {
                            eprintln!("粘贴错误: {}", msg);
                        }
                        FileOperationResult::NeedsConfirmation(_) => {}
                    }
                }

                // 处理菜单栏的重命名请求
                if menu_should_rename {
                    if let Some(ref path) = self.selected_file {
                        self.rename_input = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string();
                        self.show_rename_dialog = true;
                    }
                }

                // 处理菜单栏的删除请求
                if menu_should_delete {
                    if let Some(ref path) = self.selected_file {
                        match self.file_operations.delete_files(&[path.clone()]) {
                            FileOperationResult::NeedsConfirmation(message) => {
                                self.delete_confirmation_message = message;
                                self.show_delete_confirmation = true;
                            }
                            FileOperationResult::Error(msg) => {
                                eprintln!("删除错误: {}", msg);
                            }
                            FileOperationResult::Success => {}
                        }
                    }
                }

                // 处理菜单栏的新建文件夹请求
                if menu_should_create_folder {
                    self.new_folder_name = generate_default_folder_name(&self.current_path);
                    self.show_new_folder_dialog = true;
                }

                ui.separator();

                // 盘符栏 - 先保存当前工作区状态
                self.drive_bar.save_workspace_state(
                    &self.current_path,
                    &self.directory_current_path,
                    &self.nav_history,
                    self.history_pos
                );

                let workspace_switched = self.drive_bar.show(ui, &mut self.current_path);
                if workspace_switched {
                    // 工作区切换，恢复新工作区的状态
                    if let Some(workspace) = self.drive_bar.get_current_workspace(&self.current_path) {
                        self.current_path = workspace.current_path.clone();
                        self.directory_current_path = workspace.directory_path.clone();
                        self.nav_history = workspace.nav_history.clone();
                        self.history_pos = workspace.history_pos;

                        // 刷新两个列表
                        self.refresh_file_list();
                        self.refresh_directory_list();
                    }
                }

                ui.separator();

                // 工具栏
                let (toolbar_needs_refresh, toolbar_should_create_folder) = toolbar::show_toolbar(ui, &mut self.current_path, &mut self.view_mode);
                if toolbar_needs_refresh {
                    // 工具栏只影响内容框，不影响目录框
                    self.refresh_file_list();
                }

                // 处理新建文件夹请求
                if toolbar_should_create_folder {
                    self.new_folder_name = generate_default_folder_name(&self.current_path);
                    self.show_new_folder_dialog = true;
                }

                ui.separator();

                // 贯穿式标题栏（目录/导航/预览）
                {
                    let total_w = ui.available_width();
                    let row_h = ui.spacing().interact_size.y * 1.1;
                    let (rect, _resp) = ui.allocate_exact_size([total_w, row_h].into(), egui::Sense::hover());
                    let left_w = total_w * self.left_ratio;
                    let mid_w = total_w * self.mid_ratio;
                    let right_w = total_w - left_w - mid_w;

                    let spacing = ui.spacing().item_spacing.x;
                    let button_w = (mid_w - 3.0 * spacing) / 4.0;
                    let button_h = row_h * 0.9;

                    let font_id = ui.style().text_styles.get(&egui::TextStyle::Heading).cloned().unwrap_or_else(|| egui::FontId::default());
                    let color = ui.visuals().text_color();

                    // 左侧：目录
                    let left_rect = egui::Rect::from_min_max(egui::pos2(rect.left(), rect.top()), egui::pos2(rect.left() + left_w, rect.bottom()));
                    ui.painter().with_clip_rect(left_rect).text(egui::pos2(left_rect.left() + 6.0, left_rect.center().y), egui::Align2::LEFT_CENTER, "目录", font_id.clone(), color);

                    // 中间：四个导航按钮（与下方三栏的item_spacing保持一致）
                    let mid_left = left_rect.right() + spacing;
                    let mid_rect = egui::Rect::from_min_max(egui::pos2(mid_left, rect.top()), egui::pos2(mid_left + mid_w, rect.bottom()));
                    let mut x = mid_rect.left();
                    let make_rect = |x0: f32| egui::Rect::from_min_max(egui::pos2(x0, mid_rect.top()), egui::pos2(x0 + button_w, mid_rect.bottom()));
                    let r_back = make_rect(x);
                    let resp_back = ui.put(r_back, egui::Button::new("返回").min_size(egui::vec2(button_w, button_h)));
                    if resp_back.clicked() { self.go_back(); }
                    x += button_w + spacing;
                    let r_fwd = make_rect(x);
                    let resp_fwd = ui.put(r_fwd, egui::Button::new("前进").min_size(egui::vec2(button_w, button_h)));
                    if resp_fwd.clicked() { self.go_forward(); }
                    x += button_w + spacing;
                    let r_refresh = make_rect(x);
                    let resp_refresh = ui.put(r_refresh, egui::Button::new("刷新").min_size(egui::vec2(button_w, button_h)));
                    if resp_refresh.clicked() { self.refresh_file_list(); }
                    x += button_w + spacing;
                    let r_home = make_rect(x);
                    let resp_home = ui.put(r_home, egui::Button::new("主页").min_size(egui::vec2(button_w, button_h)));
                    if resp_home.clicked() {
                        if let Some(home_dir) = dirs::home_dir() {
                            self.current_path = home_dir.clone();
                            self.refresh_file_list();
                            self.push_history(home_dir);
                        }
                    }

                    // 右侧：预览（考虑与中栏的间距对齐）
                    let right_left = mid_rect.right() + spacing;
                    let right_rect = egui::Rect::from_min_max(egui::pos2(right_left, rect.top()), egui::pos2(rect.right(), rect.bottom()));
                    ui.painter().with_clip_rect(right_rect).text(egui::pos2(right_rect.left() + 6.0, right_rect.center().y), egui::Align2::LEFT_CENTER, "预览", font_id, color);
                }

                // 统一分割线
                ui.separator();

                // 主内容区域 - 使用剩余的全部高度
                let available_height = ui.available_height() - 40.0; // 留一些边距
                ui.horizontal(|ui| {
                    let total_w = ui.available_width();
                    let left_w = total_w * self.left_ratio;
                    let mid_w = total_w * self.mid_ratio;
                    let right_w = total_w - left_w - mid_w;
                    // 左侧目录列表 (25%宽度) - 使用FileList
                    ui.allocate_ui_with_layout(
                        [left_w, available_height].into(),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            // 左侧标题由贯穿式标题栏提供

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
                                // 确保目录框的纹理已加载
                                self.directory_list.ensure_textures(ui.ctx());

                                let (should_refresh_content, should_navigate_directory, should_open_file) =
                                    self.directory_list.show_for_directory(ui, &mut temp_current_path, &mut self.selected_file);

                                if should_refresh_content {
                                    // 单击目录：内容框刷新到该目录
                                    if let Some(selected_path) = self.selected_file.clone() {
                                        self.current_path = selected_path.clone();
                                        self.refresh_file_list();
                                        self.push_history(selected_path);
                                    }
                                }

                                if should_navigate_directory {
                                    // 双击目录：目录框进入该目录
                                    self.directory_current_path = temp_current_path.clone();
                                    self.refresh_directory_list();
                                }

                                if should_open_file {
                                    // 双击文件：文件已通过mouse_strategy打开
                                    // 这里可以添加成功打开的提示，如果需要的话
                                }
                            });
                        }
                    );

                    // 中间文件列表 (45%宽度)
                    ui.allocate_ui_with_layout(
                        [mid_w, available_height].into(),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            // 中间标题由贯穿式标题栏提供

                            let button_h = ui.spacing().interact_size.y * 1.5;
                            let total_w = ui.available_width();
                            let spacing = ui.spacing().item_spacing.x;
                            let button_w = (total_w - 3.0 * spacing) / 4.0;
                            ui.horizontal(|ui| {
                                // 复制按钮
                                if ui.add(egui::Button::new("复制").min_size(egui::vec2(button_w, button_h))).clicked() {
                                    if let Some(ref path) = self.selected_file {
                                        self.file_operations.copy_to_clipboard(vec![path.clone()]);
                                    }
                                }

                                // 粘贴按钮
                                if ui.add(egui::Button::new("粘贴").min_size(egui::vec2(button_w, button_h))).clicked() {
                                    // 总是粘贴到当前路径（内容框的当前目录）
                                    match self.file_operations.paste_from_clipboard(&self.current_path) {
                                        FileOperationResult::Success => {
                                            self.refresh_file_list();
                                        }
                                        FileOperationResult::Error(msg) => {
                                            // TODO: 显示错误消息
                                            eprintln!("粘贴错误: {}", msg);
                                        }
                                        FileOperationResult::NeedsConfirmation(_) => {}
                                    }
                                }

                                // 重命名按钮
                                if ui.add(egui::Button::new("重命名").min_size(egui::vec2(button_w, button_h))).clicked() {
                                    if let Some(ref path) = self.selected_file {
                                        self.rename_input = path.file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("")
                                            .to_string();
                                        self.show_rename_dialog = true;
                                    }
                                }

                                // 删除按钮
                                if ui.add(egui::Button::new("删除").min_size(egui::vec2(button_w, button_h))).clicked() {
                                    if let Some(ref path) = self.selected_file {
                                        match self.file_operations.delete_files(&[path.clone()]) {
                                            FileOperationResult::NeedsConfirmation(message) => {
                                                self.delete_confirmation_message = message;
                                                self.show_delete_confirmation = true;
                                            }
                                            FileOperationResult::Error(msg) => {
                                                eprintln!("删除错误: {}", msg);
                                            }
                                            FileOperationResult::Success => {
                                                // 这个情况不应该发生，删除总是需要确认
                                            }
                                        }
                                    }
                                }
                            });

                            // 独立的滚动区域
                            egui::ScrollArea::vertical().id_salt("file_scroll").show(ui, |ui| {
                                let should_navigate = self.file_list.show(ui, &mut self.current_path, &mut self.selected_file, self.view_mode);
                                if should_navigate {
                                    // 内容框点击文件夹时：只更新内容框，不刷新目录框
                                    self.current_path = self.selected_file.as_ref().unwrap_or(&self.current_path).clone();
                                    self.refresh_file_list();
                                    self.push_history(self.current_path.clone());

                                    // 目录框保持不变，不自动更新
                                }
                            });
                        }
                    );

                    // 右侧预览面板 (30%宽度)
                    ui.allocate_ui_with_layout(
                        [right_w, available_height].into(),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            // 右侧标题由贯穿式标题栏提供
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

        // 显示重命名对话框
        if self.show_rename_dialog {
            let mut open = true;
            egui::Window::new("重命名")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("新名称:");
                        ui.text_edit_singleline(&mut self.rename_input);
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("确定").clicked() {
                            if let Some(ref path) = self.selected_file {
                                match self.file_operations.rename_file(path, &self.rename_input) {
                                    FileOperationResult::Success => {
                                        self.refresh_file_list();
                                        self.show_rename_dialog = false;
                                    }
                                    FileOperationResult::Error(msg) => {
                                        eprintln!("重命名错误: {}", msg);
                                        // TODO: 显示错误消息给用户
                                    }
                                    FileOperationResult::NeedsConfirmation(_) => {}
                                }
                            }
                        }
                        if ui.button("取消").clicked() {
                            self.show_rename_dialog = false;
                        }
                    });
                });

            if !open {
                self.show_rename_dialog = false;
            }
        }

        // 显示删除确认对话框
        if self.show_delete_confirmation {
            let mut open = true;
            egui::Window::new("确认删除")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label(&self.delete_confirmation_message);
                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("确定").clicked() {
                            if let Some(ref path) = self.selected_file {
                                match self.file_operations.confirm_delete(&[path.clone()]) {
                                    FileOperationResult::Success => {
                                        self.selected_file = None;
                                        self.refresh_file_list();
                                        self.show_delete_confirmation = false;
                                    }
                                    FileOperationResult::Error(msg) => {
                                        eprintln!("删除错误: {}", msg);
                                        self.show_delete_confirmation = false;
                                    }
                                    FileOperationResult::NeedsConfirmation(_) => {}
                                }
                            }
                        }
                        if ui.button("取消").clicked() {
                            self.show_delete_confirmation = false;
                        }
                    });
                });

            if !open {
                self.show_delete_confirmation = false;
            }
        }

        // 显示新建文件夹对话框
        if self.show_new_folder_dialog {
            let mut open = true;
            egui::Window::new("新建文件夹")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("文件夹名称:");
                        ui.text_edit_singleline(&mut self.new_folder_name);
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("确定").clicked() {
                            match self.create_operations.create_folder(&self.current_path, &self.new_folder_name) {
                                CreateOperationResult::Success => {
                                    self.refresh_file_list();
                                    self.show_new_folder_dialog = false;
                                }
                                CreateOperationResult::Error(msg) => {
                                    eprintln!("新建文件夹错误: {}", msg);
                                    // TODO: 显示错误消息给用户
                                }
                                CreateOperationResult::NeedsConfirmation(_) => {}
                                CreateOperationResult::NeedsInput(_) => {}
                            }
                        }
                        if ui.button("取消").clicked() {
                            self.show_new_folder_dialog = false;
                        }
                    });
                });

            if !open {
                self.show_new_folder_dialog = false;
            }
        }

        // 显示帮助系统对话框（关于对话框等）
        if self.help_system.is_about_dialog_showing() {
            self.help_system.show_about_dialog(ctx);
        }
    }
}