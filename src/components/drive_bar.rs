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
        println!("盘符栏: 查找路径 {} 的盘符根目录", path.display());
        for drive in &self.drives {
            if path.starts_with(&drive.path) {
                println!("盘符栏: 找到匹配盘符 {}", drive.path.display());
                return drive.path.clone();
            }
        }
        println!("盘符栏: 没有找到匹配盘符，使用根目录");
        PathBuf::from("/")
    }

    pub fn show(&mut self, ui: &mut egui::Ui, current_path: &mut PathBuf) -> bool {
        // 调试：显示当前保存的路径状态
        if self.saved_paths.len() > 0 {
            println!("盘符栏: 当前保存的工作区路径:");
            for (drive_root, saved_path) in &self.saved_paths {
                println!("  {} -> {}", drive_root.display(), saved_path.display());
            }
        }
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
                    println!("盘符栏: 点击了盘符 {}", drive.path.display());
                    println!("盘符栏: 切换前的当前路径 {}", current_path.display());

                    // 简单直接：保存当前绝对路径到对应的盘符
                    // 找到当前路径属于哪个盘符
                    let mut current_drive = PathBuf::from("/");
                    for d in &self.drives {
                        if current_path.starts_with(&d.path) && d.path.as_os_str().len() > current_drive.as_os_str().len() {
                            current_drive = d.path.clone();
                        }
                    }

                    println!("盘符栏: 保存路径 {} 到盘符 {}", current_path.display(), current_drive.display());
                    self.saved_paths.insert(current_drive.clone(), current_path.clone());

                    // 切换到新盘符，恢复保存的路径
                    if let Some(saved_path) = self.saved_paths.get(&drive.path) {
                        println!("盘符栏: 恢复保存的路径 {}", saved_path.display());
                        *current_path = saved_path.clone();
                    } else {
                        println!("盘符栏: 没有保存的路径，使用盘符根目录 {}", drive.path.display());
                        *current_path = drive.path.clone();
                    }

                    println!("盘符栏: 切换后的路径 {}", current_path.display());
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