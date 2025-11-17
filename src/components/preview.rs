use eframe::egui;
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::utils;
use image::GenericImageView;

pub struct Preview {
    current_file: Option<PathBuf>,
    preview_content: String,
    file_info: FileInfo,
    image_texture: Option<egui::TextureHandle>,
    image_size: Option<(u32, u32)>,
    // 图片缓存
    texture_cache: HashMap<String, CachedImage>,
    // 性能优化：加载状态
    is_loading: bool,
    pending_file: Option<PathBuf>,
    // 异步加载
    loading_result: Option<Arc<Mutex<Option<LoadingResult>>>>,
}

struct LoadingResult {
    img_rgba: Option<image::RgbaImage>,
    size: Option<(u32, u32)>,
    error: Option<String>,
    file_path: PathBuf,
}

struct CachedImage {
    texture: egui::TextureHandle,
    size: (u32, u32),
    file_size: u64,
    last_modified: std::time::SystemTime,
}

#[derive(Default)]
struct FileInfo {
    size: String,
    modified: String,
    file_type: String,
}

impl Preview {
    pub fn new() -> Self {
        Self {
            current_file: None,
            preview_content: String::new(),
            file_info: FileInfo::default(),
            image_texture: None,
            image_size: None,
            texture_cache: HashMap::new(),
            is_loading: false,
            pending_file: None,
            loading_result: None,
        }
    }

    pub fn clear(&mut self) {
        self.current_file = None;
        self.preview_content.clear();
        self.file_info = FileInfo::default();
        self.image_texture = None;
        self.image_size = None;
        self.is_loading = false;
        self.pending_file = None;
        self.loading_result = None;
        // 清理缓存但保留最近的几个以提高性能
        self.cleanup_cache();
    }

    pub fn load_preview(&mut self, path: PathBuf, ctx: &egui::Context) {
        if self.current_file.as_ref() == Some(&path) && !self.is_loading {
            return;
        }

        // 如果当前正在加载其他文件，取消并加载新的
        if self.is_loading {
            self.pending_file = Some(path.clone());
            return;
        }

        self.current_file = Some(path.clone());
        self.preview_content.clear();
        self.image_texture = None;
        self.image_size = None;
        self.is_loading = false;

        // 检查是否为文件夹
        if path.is_dir() {
            // 使用原有的文件夹预览逻辑
            self.generate_folder_preview(&path);
        } else {
            // 检查文件类型
            match path.extension().and_then(|ext| ext.to_str()) {
                Some("txt") | Some("rs") | Some("js") | Some("py") | Some("html") |
                Some("css") | Some("json") | Some("xml") | Some("md") => {
                    // 文本文件预览
                    self.generate_text_preview(&path);
                }
                Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("bmp") => {
                    // 图片文件预览
                    // 先检查缓存，如果有就直接显示
                    if let Some((texture, size)) = self.get_cached_image(&path) {
                        self.image_texture = Some(texture);
                        self.image_size = Some(size);
                        self.preview_content = format!(
                            "图片预览 (已缓存)\n\n尺寸: {} x {} 像素\n格式: {}",
                            size.0,
                            size.1,
                            path.extension()
                                .and_then(|ext| ext.to_str())
                                .map(|ext| ext.to_uppercase())
                                .unwrap_or_else(|| "未知".to_string())
                        );
                        self.is_loading = false;
                    } else {
                        // 没有缓存，启动异步加载
                        self.is_loading = true;
                        self.preview_content = "正在加载图片...".to_string();
                        self.start_async_loading(path.clone(), ctx.clone());
                    }
                }
                _ => {
                    // 其他文件类型
                    self.preview_content = "此文件类型不支持预览".to_string();
                }
            }
        }

        // 获取文件信息
        if let Ok(metadata) = fs::metadata(&path) {
            self.file_info.size = utils::get_file_size_str(metadata.len());
            self.file_info.modified = utils::get_file_modified_time(&path)
                .unwrap_or_else(|| "未知时间".to_string());
        }

        self.file_info.file_type = self.get_file_type(&path);
    }

    // 在每帧更新时调用，用于处理异步加载结果
    pub fn update(&mut self, ctx: &egui::Context) {
        // 先获取需要的信息，避免借用冲突
        let (should_process, current_file, size, error) = {
            let mut found = false;
            let mut cur_file = PathBuf::new();
            let mut img_size = None;
            let mut img_error = None;

            if self.is_loading {
                if let Some(loading_result) = &self.loading_result {
                    if let Ok(result_guard) = loading_result.lock() {
                        if let Some(result) = result_guard.as_ref() {
                            // 检查结果是否对应当前文件
                            if let Some(current_file) = &self.current_file {
                                if result.file_path == *current_file {
                                    found = true;
                                    cur_file = current_file.clone();
                                    img_size = result.size;
                                    img_error = result.error.clone();
                                }
                            }
                        }
                    }
                }
            }

            (found, cur_file, img_size, img_error)
        };

        // 处理结果 - 直接访问loading_result中的数据避免拷贝
        if should_process {
            if let Some(loading_result) = &self.loading_result {
                if let Ok(result_guard) = loading_result.lock() {
                    if let Some(result) = result_guard.as_ref() {
                        if let Some(img_rgba) = &result.img_rgba {
                            if let Some((width, height)) = result.size {
                                // 从RgbaImage创建ColorImage，避免额外的数据拷贝
                                let img_size = [img_rgba.width() as usize, img_rgba.height() as usize];
                                let color_image = egui::ColorImage::from_rgba_premultiplied(img_size, img_rgba);

                                let texture = ctx.load_texture(
                                    format!("async_image_{}", current_file.display()),
                                    color_image,
                                    egui::TextureOptions::default(),
                                );

                                self.image_texture = Some(texture);
                                self.image_size = Some((width, height));

                                self.preview_content = format!(
                                    "图片预览\n\n尺寸: {} x {} 像素\n格式: {}",
                                    width,
                                    height,
                                    current_file.extension()
                                        .and_then(|ext| ext.to_str())
                                        .map(|ext| ext.to_uppercase())
                                        .unwrap_or_else(|| "未知".to_string())
                                );
                            }
                        } else if let Some(error) = &result.error {
                            self.preview_content = format!("无法加载图片: {}", error);
                            self.image_texture = None;
                            self.image_size = None;
                        }

                        self.is_loading = false;
                    }
                }
            }
        }

        // 检查是否有待处理的文件
        if !self.is_loading {
            if let Some(pending) = self.pending_file.take() {
                self.load_preview(pending, ctx);
            }
        }
    }

    fn get_file_type(&self, path: &Path) -> String {
        if path.is_dir() {
            "文件夹".to_string()
        } else {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_uppercase())
                .unwrap_or_else(|| "文件".to_string())
        }
    }

    fn generate_preview(&mut self, path: &Path, ctx: &egui::Context) {
        if path.is_dir() {
            self.generate_folder_preview(path);
        } else {
            match path.extension().and_then(|ext| ext.to_str()) {
                Some("txt") | Some("rs") | Some("js") | Some("py") | Some("html") |
                Some("css") | Some("json") | Some("xml") | Some("md") => {
                    self.generate_text_preview(path);
                }
                Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("bmp") => {
                    self.generate_preview(path, ctx);
                }
                _ => {
                    self.preview_content = "此文件类型不支持预览".to_string();
                }
            }
        }
    }

    fn generate_folder_preview(&mut self, path: &Path) {
        if let Ok(entries) = fs::read_dir(path) {
            let mut folders = Vec::new();
            let mut files = Vec::new();

            for entry in entries.flatten() {
                let entry_path = entry.path();
                let name = entry_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("未知")
                    .to_string();

                if entry_path.is_dir() {
                    folders.push(name);
                } else {
                    files.push(name);
                }
            }

            self.preview_content = format!(
                "文件夹内容 ({} 个文件夹, {} 个文件)\n\n📁 文件夹:\n{}\n\n📄 文件:\n{}",
                folders.len(),
                files.len(),
                folders.iter().take(20).map(|f| format!("  {}", f)).collect::<Vec<_>>().join("\n"),
                files.iter().take(20).map(|f| format!("  {}", f)).collect::<Vec<_>>().join("\n")
            );

            if folders.len() > 20 || files.len() > 20 {
                self.preview_content.push_str("\n\n... 还有更多项目");
            }
        } else {
            self.preview_content = "无法读取文件夹内容".to_string();
        }
    }

    fn generate_text_preview(&mut self, path: &Path) {
        if let Ok(content) = fs::read_to_string(path) {
            // 限制预览长度
            let lines: Vec<&str> = content.lines().collect();
            let preview_lines = lines.iter().take(100).collect::<Vec<_>>();

            self.preview_content = if lines.len() > 100 {
                format!(
                    "文本预览 (前100行，共{}行):\n\n{}",
                    lines.len(),
                    preview_lines.iter().map(|&&line| line).collect::<Vec<_>>().join("\n")
                )
            } else {
                format!(
                    "文本预览 ({}行):\n\n{}",
                    lines.len(),
                    preview_lines.iter().map(|&&line| line).collect::<Vec<_>>().join("\n")
                )
            };
        } else {
            self.preview_content = "无法读取文件内容".to_string();
        }
    }

    
    pub fn show(&mut self, ui: &mut egui::Ui) {
        if let Some(path) = &self.current_file {
            ui.vertical(|ui| {
                // 文件信息
                ui.group(|ui| {
                    ui.heading("文件信息");
                    ui.label(format!("名称: {}", path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("未知文件")));
                    ui.label(format!("类型: {}", self.file_info.file_type));
                    ui.label(format!("大小: {}", self.file_info.size));
                    ui.label(format!("修改时间: {}", self.file_info.modified));
                });

                ui.separator();

                // 预览内容
                if let Some(texture) = &self.image_texture {
                    // 显示图片
                    ui.vertical(|ui| {
                        ui.label("图片预览:");

                        // 检查纹理尺寸是否有效
                        let texture_size = texture.size();
                        if texture_size[0] > 0 && texture_size[1] > 0 {
                            // 限制最大显示尺寸
                            let max_size = ui.available_size() - egui::vec2(20.0, 20.0);
                            let mut image_size = egui::vec2(texture_size[0] as f32, texture_size[1] as f32);

                            // 缩放图片以适应可用空间
                            let scale = (max_size.x / image_size.x).min(max_size.y / image_size.y).min(1.0);
                            image_size *= scale;

                            // 确保缩放后的尺寸是有效的
                            if image_size.x > 0.0 && image_size.y > 0.0 {
                                let result = ui.add(
                                    egui::Image::from_texture(egui::load::SizedTexture::new(
                                        texture.id(),
                                        image_size,
                                    ))
                                );

                                // 如果图片渲染出错，显示错误信息
                                if result.hovered() {
                                    ui.label("图片渲染正常");
                                }
                            } else {
                                ui.label("图片尺寸无效");
                            }

                            // 显示图片信息
                            if let Some((width, height)) = self.image_size {
                                ui.label(format!("实际尺寸: {} x {} 像素", width, height));
                                ui.label(format!("显示尺寸: {:.0} x {:.0} 像素", image_size.x, image_size.y));
                            }
                        } else {
                            ui.label("纹理数据无效");
                        }
                    });
                } else if !self.preview_content.is_empty() {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.monospace(&self.preview_content);
                    });
                } else {
                    ui.label("无预览内容");
                }
            });
        } else {
            ui.label("选择一个文件查看预览");
        }
    }

    // 缓存管理方法
    fn get_cache_key(&self, path: &Path) -> String {
        let modified_time = path.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        format!("{}_{:?}", path.to_string_lossy(), modified_time)
    }

    fn is_cache_valid(&self, path: &Path, cached: &CachedImage) -> bool {
        if let Ok(metadata) = path.metadata() {
            if let Ok(modified) = metadata.modified() {
                return cached.file_size == metadata.len() && cached.last_modified == modified;
            }
        }
        false
    }

    fn cleanup_cache(&mut self) {
        // 保留最近10个图片的缓存，删除其他
        if self.texture_cache.len() > 10 {
            let mut keys: Vec<_> = self.texture_cache.keys().cloned().collect();
            keys.sort(); // 简单的字符串排序，实际项目中可能需要更复杂的策略

            for key in keys.iter().take(self.texture_cache.len() - 10) {
                self.texture_cache.remove(key);
            }
        }
    }

    fn get_cached_image(&self, path: &Path) -> Option<(egui::TextureHandle, (u32, u32))> {
        let cache_key = self.get_cache_key(path);
        if let Some(cached) = self.texture_cache.get(&cache_key) {
            if self.is_cache_valid(path, cached) {
                return Some((cached.texture.clone(), cached.size));
            }
        }
        None
    }

    fn cache_image(&mut self, path: &Path, texture: egui::TextureHandle, size: (u32, u32)) {
        let cache_key = self.get_cache_key(path);
        if let Ok(metadata) = path.metadata() {
            if let Ok(modified) = metadata.modified() {
                let cached = CachedImage {
                    texture,
                    size,
                    file_size: metadata.len(),
                    last_modified: modified,
                };
                self.texture_cache.insert(cache_key, cached);
                self.cleanup_cache();
            }
        }
    }

    // 异步图片加载
    fn start_async_loading(&mut self, path: PathBuf, ctx: egui::Context) {
        let result_arc: Arc<Mutex<Option<LoadingResult>>> = Arc::new(Mutex::new(None));
        self.loading_result = Some(result_arc.clone());

        // 克隆必要的变量到线程中
        let path_clone = path.clone();
        let ctx_clone = ctx.clone();

        // 启动后台线程进行图片加载
        thread::spawn(move || {
            let loading_result = Self::load_image_in_background(&path_clone, &ctx_clone);

            // 将结果写入共享内存
            if let Ok(mut result_guard) = result_arc.lock() {
                *result_guard = Some(loading_result);
            }

            // 请求重绘UI
            ctx_clone.request_repaint();
        });
    }

    // 在后台线程中加载图片
    fn load_image_in_background(path: &Path, _ctx: &egui::Context) -> LoadingResult {
        // 检查是否为目录
        if path.is_dir() {
            return LoadingResult {
                img_rgba: None,
                size: None,
                error: Some("这是一个文件夹，不是图片文件".to_string()),
                file_path: path.to_path_buf(),
            };
        }

        // 检查文件是否存在
        if !path.exists() {
            return LoadingResult {
                img_rgba: None,
                size: None,
                error: Some("文件不存在".to_string()),
                file_path: path.to_path_buf(),
            };
        }

        // 检查文件扩展名是否为图片格式
        let is_image = path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "gif" | "bmp"))
            .unwrap_or(false);

        if !is_image {
            return LoadingResult {
                img_rgba: None,
                size: None,
                error: Some("文件不是支持的图片格式".to_string()),
                file_path: path.to_path_buf(),
            };
        }

        // 检查文件大小
        if let Ok(metadata) = fs::metadata(path) {
            let file_size_bytes = metadata.len();
            if file_size_bytes > 50 * 1024 * 1024 {
                return LoadingResult {
                    img_rgba: None,
                    size: None,
                    error: Some(format!("图片文件过大 ({} MB)", file_size_bytes / (1024 * 1024))),
                    file_path: path.to_path_buf(),
                };
            }
        }

        // 加载图片
        match image::open(path) {
            Ok(img) => {
                let (width, height) = img.dimensions();

                // 检查图片尺寸
                if width > 8192 || height > 8192 {
                    return LoadingResult {
                        img_rgba: None,
                        size: Some((width, height)),
                        error: Some(format!("图片尺寸过大 ({} x {})", width, height)),
                        file_path: path.to_path_buf(),
                    };
                }

                // 使用更高效的RGBA转换，避免重复调用
                let img_rgba = img.to_rgba8();
                let size = [width as usize, height as usize];

                // 检查图片数据大小
                let expected_size = size[0] * size[1] * 4;
                if expected_size > 100 * 1024 * 1024 {
                    return LoadingResult {
                        img_rgba: None,
                        size: Some((width, height)),
                        error: Some("图片数据量过大".to_string()),
                        file_path: path.to_path_buf(),
                    };
                }

                // 直接返回RgbaImage，在主线程中创建ColorImage
                LoadingResult {
                    img_rgba: Some(img_rgba),
                    size: Some((width, height)),
                    error: None,
                    file_path: path.to_path_buf(),
                }
            }
            Err(e) => {
                LoadingResult {
                    img_rgba: None,
                    size: None,
                    error: Some(format!("无法加载图片: {}", e)),
                    file_path: path.to_path_buf(),
                }
            }
        }
    }
}