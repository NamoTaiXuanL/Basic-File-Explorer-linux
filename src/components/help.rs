use eframe::egui;

// 帮助系统
pub struct HelpSystem {
    show_about_dialog: bool,
}

impl HelpSystem {
    pub fn new() -> Self {
        Self {
            show_about_dialog: false,
        }
    }

    // 显示关于对话框
    pub fn show_about_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;

        egui::Window::new("关于文件浏览器")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .default_size(egui::Vec2::new(400.0, 300.0))
            .open(&mut open)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    // 应用图标和名称
                    ui.add_space(10.0);
                    ui.heading("📁 文件浏览器");
                    ui.add_space(5.0);
                    ui.label("版本 1.0.0");
                    ui.add_space(20.0);

                    // 项目信息
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("项目组:");
                            ui.label("lilith 项目组");
                        });
                        ui.horizontal(|ui| {
                            ui.label("开发者:");
                            ui.label("Seraphiel");
                        });
                        ui.horizontal(|ui| {
                            ui.label("邮箱:");
                            ui.hyperlink_to("leeking666888@gmail.com", "mailto:leeking666888@gmail.com");
                        });
                    });

                    ui.add_space(20.0);

                    // 功能说明
                    ui.group(|ui| {
                        ui.label("主要功能:");
                        ui.label("• 文件和文件夹浏览");
                        ui.label("• 复制、粘贴、重命名、删除操作");
                        ui.label("• 新建文件夹功能");
                        ui.label("• 隐藏文件显示切换");
                        ui.label("• 文件预览功能");
                    });

                    ui.add_space(20.0);

                    // 技术信息
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("技术栈:");
                            ui.label("Rust + egui");
                        });
                        ui.horizontal(|ui| {
                            ui.label("许可证:");
                            ui.label("MIT License");
                        });
                    });

                    ui.add_space(20.0);

                    // 版权信息
                    ui.label("© 2025 lilith 项目组. 保留所有权利.");
                });

                ui.separator();
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("确定").clicked() {
                            self.show_about_dialog = false;
                        }
                    });
                });
            });

        if !open {
            self.show_about_dialog = false;
        }
    }

    // 触发显示关于对话框
    pub fn show_about(&mut self) {
        self.show_about_dialog = true;
    }

    // 检查是否正在显示关于对话框
    pub fn is_about_dialog_showing(&self) -> bool {
        self.show_about_dialog
    }
}

