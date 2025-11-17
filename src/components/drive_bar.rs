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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceState {
    pub current_path: PathBuf,      // 内容框的当前路径
    pub directory_path: PathBuf,     // 目录框的当前路径
    pub nav_history: Vec<PathBuf>,   // 导航历史
    pub history_pos: usize,          // 历史位置
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

#[derive(Serialize, Deserialize)]
struct WorkspaceCacheData {
    workspaces: Vec<(char, WorkspaceState)>,
    timestamp: u64,
}

pub struct DriveBar {
    drives: Vec<DriveInfo>,
    workspaces: std::collections::HashMap<char, WorkspaceState>,
    cache_file: String,
    workspace_cache_file: String,
}

impl DriveBar {
    const CACHE_EXPIRE_SECONDS: u64 = 3600; // 缓存1小时过期

    pub fn new(initial_path: &PathBuf) -> Self {
        let cache_file = "drives_cache.json".to_string();
        let workspace_cache_file = "workspaces_cache.json".to_string();
        let mut drive_bar = Self {
            drives: Vec::new(),
            workspaces: std::collections::HashMap::new(),
            cache_file,
            workspace_cache_file,
        };

        // 尝试加载缓存，如果失败则刷新
        if !drive_bar.load_from_cache() {
            drive_bar.refresh_drives();
        }

        // 加载工作区状态
        drive_bar.load_workspaces_from_cache();

        // 为当前路径初始化工作区
        if let Some(drive_letter) = Self::get_drive_letter_from_path(initial_path) {
            drive_bar.ensure_workspace_exists(drive_letter, initial_path);
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
                    let real_label = Self::get_drive_volume_label(&drive_path);
                    let drive_info = DriveInfo {
                        letter: char::from(letter),
                        label: real_label,
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

    // 获取磁盘的真实卷标
    #[cfg(target_os = "windows")]
    fn get_drive_volume_label(drive_path: &PathBuf) -> String {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        use winapi::um::fileapi::{GetVolumeInformationW};
        use std::ptr;

        let path_str = format!("{}\\", drive_path.to_string_lossy());
        let mut wide_path: Vec<u16> = path_str.encode_utf16().chain(std::iter::once(0)).collect();

        let mut volume_name_buffer = [0u16; 256];
        let mut file_system_name_buffer = [0u16; 256];

        unsafe {
            let result = GetVolumeInformationW(
                wide_path.as_ptr(),
                volume_name_buffer.as_mut_ptr(),
                volume_name_buffer.len() as u32,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                file_system_name_buffer.as_mut_ptr(),
                file_system_name_buffer.len() as u32,
            );

            if result != 0 {
                let label = OsString::from_wide(&volume_name_buffer[..])
                    .to_string_lossy()
                    .into_owned();

                // 如果获取到真实标签且不为空，使用真实标签
                if !label.trim().is_empty() {
                    label
                } else {
                    // 如果没有标签，使用默认格式
                    let drive_letter = drive_path.to_string_lossy();
                    if drive_letter.len() >= 1 {
                        format!("本地磁盘 ({})", drive_letter.chars().next().unwrap())
                    } else {
                        "本地磁盘".to_string()
                    }
                }
            } else {
                // 获取失败，使用默认格式
                let drive_letter = drive_path.to_string_lossy();
                if drive_letter.len() >= 1 {
                    format!("本地磁盘 ({})", drive_letter.chars().next().unwrap())
                } else {
                    "本地磁盘".to_string()
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn get_drive_volume_label(_drive_path: &PathBuf) -> String {
        // Linux/Unix系统直接返回路径作为标签
        _drive_path.to_string_lossy().into_owned()
    }

    pub fn show(&mut self, ui: &mut egui::Ui, current_path: &mut PathBuf) -> bool {
        let mut workspace_switched = false;

        // 先收集需要切换的盘符
        let mut clicked_drive = None;

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
                    clicked_drive = Some(drive.letter);
                }

                // 显示工具提示
                response.on_hover_ui(|ui| {
                    ui.label(format!("路径: {}", drive.path.display()));
                });
            }
        });

        // 在循环外执行工作区切换
        if let Some(drive_letter) = clicked_drive {
            workspace_switched = self.switch_to_workspace(drive_letter, current_path);
        }

        workspace_switched
    }

    // 保存当前工作区状态
    pub fn save_workspace_state(&mut self, current_path: &PathBuf, directory_path: &PathBuf,
                               nav_history: &[PathBuf], history_pos: usize) {
        if let Some(drive_letter) = Self::get_drive_letter_from_path(current_path) {
            let workspace = WorkspaceState {
                current_path: current_path.clone(),
                directory_path: directory_path.clone(),
                nav_history: nav_history.to_vec(),
                history_pos,
            };

            self.workspaces.insert(drive_letter, workspace);
            self.save_workspaces_to_cache();
        }
    }

    // 切换到指定盘符的工作区
    fn switch_to_workspace(&mut self, drive_letter: char, current_path: &mut PathBuf) -> bool {
        // 先保存当前工作区状态（由调用者负责）

        // 切换到目标工作区
        if let Some(workspace) = self.workspaces.get(&drive_letter) {
            *current_path = workspace.current_path.clone();
            true
        } else {
            // 创建新的工作区
            let drive_path = PathBuf::from(format!("{}:/", drive_letter));
            let workspace = WorkspaceState {
                current_path: drive_path.clone(),
                directory_path: drive_path.clone(),
                nav_history: vec![drive_path.clone()],
                history_pos: 0,
            };

            *current_path = drive_path.clone();
            self.workspaces.insert(drive_letter, workspace);
            true
        }
    }

    // 获取当前工作区状态
    pub fn get_current_workspace(&self, current_path: &PathBuf) -> Option<&WorkspaceState> {
        if let Some(drive_letter) = Self::get_drive_letter_from_path(current_path) {
            self.workspaces.get(&drive_letter)
        } else {
            None
        }
    }

    // 确保工作区存在
    fn ensure_workspace_exists(&mut self, drive_letter: char, initial_path: &PathBuf) {
        if !self.workspaces.contains_key(&drive_letter) {
            let workspace = WorkspaceState {
                current_path: initial_path.clone(),
                directory_path: initial_path.parent()
                    .unwrap_or(initial_path)
                    .to_path_buf(),
                nav_history: vec![initial_path.clone()],
                history_pos: 0,
            };
            self.workspaces.insert(drive_letter, workspace);
        }
    }

    // 从路径获取盘符
    fn get_drive_letter_from_path(path: &PathBuf) -> Option<char> {
        if let Some(path_str) = path.to_str() {
            if path_str.len() >= 2 && path_str.chars().nth(1) == Some(':') {
                path_str.chars().next()
            } else {
                None
            }
        } else {
            None
        }
    }

    // 加载工作区缓存
    fn load_workspaces_from_cache(&mut self) {
        if let Ok(content) = fs::read_to_string(&self.workspace_cache_file) {
            if let Ok(cache_data) = serde_json::from_str::<WorkspaceCacheData>(&content) {
                // 检查缓存是否过期（工作区缓存保留24小时）
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                if current_time - cache_data.timestamp < 86400 { // 24小时
                    self.workspaces = cache_data.workspaces.into_iter().collect();
                }
            }
        }
    }

    // 保存工作区缓存
    fn save_workspaces_to_cache(&self) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let workspaces_vec: Vec<(char, WorkspaceState)> = self.workspaces
            .iter()
            .map(|(k, v)| (*k, v.clone()))
            .collect();

        let cache_data = WorkspaceCacheData {
            workspaces: workspaces_vec,
            timestamp: current_time,
        };

        if let Ok(content) = serde_json::to_string_pretty(&cache_data) {
            let _ = fs::write(&self.workspace_cache_file, content);
        }
    }
}