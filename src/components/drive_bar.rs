use eframe::egui;
use std::path::PathBuf;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveInfo {
    pub letter: char,
    pub label: String,
    #[serde(with = "serde_path")]
    pub path: PathBuf,
}

// 用于PathBuf序列化的辅助模块
mod serde_path {
    use std::path::PathBuf;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(path: &PathBuf, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let path_str = path.to_string_lossy().into_owned();
        serializer.serialize_str(&path_str)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
    where
        D: Deserializer<'de>,
    {
        let path_str = String::deserialize(deserializer)?;
        Ok(PathBuf::from(path_str))
    }
}

#[derive(Serialize, Deserialize)]
struct CacheData {
    drives: Vec<DriveInfo>,
    timestamp: u64,
}

pub struct DriveBar {
    drives: Vec<DriveInfo>,
    cache_file: String,
}

impl DriveBar {
    const CACHE_EXPIRE_SECONDS: u64 = 3600; // 缓存1小时过期
}

impl DriveBar {
    pub fn new() -> Self {
        let cache_file = "drives_cache.json".to_string();
        let mut drive_bar = Self {
            drives: Vec::new(),
            cache_file,
        };

        // 尝试加载缓存，如果失败则刷新
        if !drive_bar.load_from_cache() {
            drive_bar.refresh_drives();
        }

        drive_bar
    }

    pub fn refresh_drives(&mut self) {
        self.drives.clear();
        self.drives = Self::get_system_drives();
        self.save_to_cache();
    }

    // 手动刷新，仅在用户主动调用时使用
    pub fn force_refresh(&mut self) {
        self.refresh_drives();
    }

    fn load_from_cache(&mut self) -> bool {
        if let Ok(content) = fs::read_to_string(&self.cache_file) {
            if let Ok(cache_data) = serde_json::from_str::<CacheData>(&content) {
                // 检查缓存是否过期
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                if current_time - cache_data.timestamp < Self::CACHE_EXPIRE_SECONDS {
                    // 缓存未过期，验证驱动器是否仍然存在
                    let valid_drives: Vec<DriveInfo> = cache_data.drives
                        .into_iter()
                        .filter(|drive| drive.path.exists())
                        .collect();

                    if !valid_drives.is_empty() {
                        self.drives = valid_drives;
                        return true;
                    }
                }
            }
        }
        false
    }

    fn save_to_cache(&self) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let cache_data = CacheData {
            drives: self.drives.clone(),
            timestamp: current_time,
        };

        if let Ok(content) = serde_json::to_string_pretty(&cache_data) {
            let _ = fs::write(&self.cache_file, content);
        }
    }

    fn get_system_drives() -> Vec<DriveInfo> {
        let mut drives = Vec::new();

        // Windows系统获取盘符
        #[cfg(target_os = "windows")]
        {
            // 检查A到Z盘符
            for letter in b'A'..=b'Z' {
                let drive_path = PathBuf::from(format!("{}:/", char::from(letter)));
                if drive_path.exists() {
                    let drive_info = DriveInfo {
                        letter: char::from(letter),
                        label: format!("本地磁盘 ({})", char::from(letter)),
                        path: drive_path,
                    };
                    drives.push(drive_info);
                }
            }
        }

        // Linux/Unix系统获取挂载点
        #[cfg(not(target_os = "windows"))]
        {
            // 常见的挂载点
            let common_mounts = vec!["/", "/home"];
            for mount in common_mounts {
                let drive_path = PathBuf::from(mount);
                if drive_path.exists() {
                    let drive_info = DriveInfo {
                        letter: mount.chars().next().unwrap_or('/'),
                        label: mount.to_string(),
                        path: drive_path,
                    };
                    drives.push(drive_info);
                }
            }
        }

        drives
    }

    pub fn show(&mut self, ui: &mut egui::Ui, current_path: &mut PathBuf) -> bool {
        let mut needs_refresh = false;

        // 显示盘符按钮栏
        ui.horizontal(|ui| {
            // 盘符标签也用按钮显示，保持对齐一致
            let label_button = egui::Button::new("盘符:")
                .min_size(egui::vec2(60.0, 24.0));
            ui.add(label_button);

            ui.separator();

            for drive in &self.drives {
                let drive_text = format!("{} ({})", drive.letter, drive.label);
                let button = egui::Button::new(drive_text)
                    .min_size(egui::vec2(100.0, 24.0));

                let response = ui.add(button);

                if response.clicked() {
                    *current_path = drive.path.clone();
                    needs_refresh = true;
                }

                // 显示工具提示
                response.on_hover_ui(|ui| {
                    ui.label(format!("路径: {}", drive.path.display()));
                });
            }
        });

        needs_refresh
    }
}