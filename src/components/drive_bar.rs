use eframe::egui;
use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Drive {
    pub path: PathBuf,
    pub name: String,
    pub is_mounted: bool,
}

pub struct DriveBar {
    drives: Vec<Drive>,
    saved_paths: HashMap<PathBuf, PathBuf>,  // 盘符路径 -> 保存的工作路径
}

impl DriveBar {
    pub fn new(current_path: &PathBuf) -> Self {
        let mut drive_bar = Self {
            drives: Vec::new(),
            saved_paths: HashMap::new(),
        };
        drive_bar.refresh_drives();
        drive_bar
    }

    fn refresh_drives(&mut self) {
        self.drives.clear();

        self.drives.push(Drive {
            path: PathBuf::from("/"),
            name: "根目录 /".to_string(),
            is_mounted: true,
        });

        self.scan_mount_points("/media");
        self.scan_mount_points("/mnt");

        let common_mounts = ["/home", "/var", "/opt", "/usr"];
        for mount in &common_mounts {
            if PathBuf::from(mount).exists() {
                self.drives.push(Drive {
                    path: PathBuf::from(mount),
                    name: format!("{} {}", mount, match *mount {
                        "/home" => "(用户目录)",
                        "/var" => "(变量数据)",
                        "/opt" => "(可选软件)",
                        "/usr" => "(用户程序)",
                        _ => "",
                    }),
                    is_mounted: true,
                });
            }
        }
    }

    fn scan_mount_points(&mut self, base_path: &str) {
        if let Ok(entries) = fs::read_dir(base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name() {
                        self.drives.push(Drive {
                            path: path.clone(),
                            name: name.to_string_lossy().to_string(),
                            is_mounted: true,
                        });
                    }
                }
            }
        }
    }

    fn find_drive_root(&self, path: &PathBuf) -> PathBuf {
        for drive in &self.drives {
            if path.starts_with(&drive.path) {
                return drive.path.clone();
            }
        }
        PathBuf::from("/")
    }

    pub fn show(&mut self, ui: &mut egui::Ui, current_path: &mut PathBuf) -> bool {
        let mut workspace_switched = false;

        ui.horizontal(|ui| {
            ui.label("盘符:");

            for drive in &self.drives {
                let is_current = current_path.starts_with(&drive.path);

                let button_text = if is_current {
                    format!("✓ {}", drive.name)
                } else {
                    drive.name.clone()
                };

                if ui.add(
                    egui::Button::new(button_text)
                        .small()
                        .fill(if is_current {
                            ui.visuals().selection.bg_fill
                        } else {
                            egui::Color32::TRANSPARENT
                        })
                ).clicked() {
                    // 保存当前路径到当前盘符
                    let current_drive_root = self.find_drive_root(current_path);
                    self.saved_paths.insert(current_drive_root, current_path.clone());

                    // 切换到新盘符，恢复保存的路径或使用盘符根目录
                    *current_path = self.saved_paths.get(&drive.path)
                        .cloned()
                        .unwrap_or_else(|| drive.path.clone());

                    workspace_switched = true;
                }
            }
        });

        workspace_switched
    }

    pub fn save_workspace_state(
        &mut self,
        current_path: &PathBuf,
        _directory_current_path: &PathBuf,
        _nav_history: &[PathBuf],
        _history_pos: usize,
    ) {
        let drive_root = self.find_drive_root(current_path);
        self.saved_paths.insert(drive_root, current_path.clone());
    }
}