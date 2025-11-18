use eframe::egui;
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::{self, Sender};
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
    // 多线程预加载
    preloader: Option<ThumbnailPreloader>,
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

// 多线程缩略图预加载器
struct ThumbnailPreloader {
    sender: mpsc::Sender<PathBuf>,
    cache: Arc<Mutex<HashMap<String, (image::RgbaImage, (u32, u32))>>>,
    _threads: Vec<thread::JoinHandle<()>>,
}

impl ThumbnailPreloader {
    fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<PathBuf>();
        let cache = Arc::new(Mutex::new(HashMap::new()));

        // 启动20个预加载线程以提高并发性能
        let mut threads = Vec::new();
        
        // 使用工作队列模式：单个消费者线程分发任务到线程池
        let cache_clone = cache.clone();
        threads.push(thread::spawn(move || {
            while let Ok(image_path) = receiver.recv() {
                let cache_clone = cache_clone.clone();
                thread::spawn(move || {
                    if let Ok(thumbnail) = Self::generate_thumbnail(&image_path) {
                        let cache_key = image_path.to_string_lossy().to_string();
                        let size = (thumbnail.width(), thumbnail.height());
                        if let Ok(mut cache_guard) = cache_clone.lock() {
                            // 缓存原始图像数据，纹理创建在主线程进行
                            cache_guard.insert(cache_key, (thumbnail, size));
                        }
                    }
                });
            }
        }));

        Self {
            sender,
            cache,
            _threads: threads,
        }
    }

    // 文件大小检查现在在工作线程中进行，避免阻塞UI

    fn get_cached_thumbnail(&self, path: &Path, ctx: &egui::Context) -> Option<(egui::TextureHandle, (u32, u32))> {
        let cache_key = path.to_string_lossy().to_string();
        if let Ok(mut cache_guard) = self.cache.lock() {
            if let Some((rgba_img, size)) = cache_guard.remove(&cache_key) {
                // 在主线程创建纹理
                let color_image = egui::ColorImage::from_rgba_premultiplied(
                    [rgba_img.width() as usize, rgba_img.height() as usize],
                    &rgba_img
                );
                let texture = ctx.load_texture(
                    format!("preloaded_{}", cache_key),
                    color_image,
                    egui::TextureOptions::default(),
                );
                Some((texture, size))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn generate_thumbnail(path: &Path) -> Result<image::RgbaImage, Box<dyn std::error::Error>> {
        let img = image::open(path)?;

        // 统一生成400px缩略图用于预加载
        let thumbnail_size = 400;
        let thumbnail = if img.width() > thumbnail_size || img.height() > thumbnail_size {
            let scale = (thumbnail_size as f32 / img.width().max(img.height()) as f32).min(1.0);
            let new_width = (img.width() as f32 * scale) as u32;
            let new_height = (img.height() as f32 * scale) as u32;

            img.resize(new_width, new_height, image::imageops::FilterType::Nearest)
        } else {
            img
        };

        Ok(thumbnail.to_rgba8())
    }
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
            preloader: None, // 稍后在第一次使用时初始化
        }
    }

    // 初始化预加载器
    pub fn init_preloader(&mut self) {
        if self.preloader.is_none() {
            self.preloader = Some(ThumbnailPreloader::new());
        }
    }

    // 预加载文件夹中的所有图片
    pub fn preload_folder_images(&mut self, folder_path: &Path) {
        if let Some(preloader) = &self.preloader {
            let preloader_clone = preloader.sender.clone();
            let folder_path = folder_path.to_path_buf();
            
            // 在后台线程中执行文件系统操作，避免阻塞UI
            thread::spawn(move || {
                if let Ok(entries) = fs::read_dir(&folder_path) {
                    let image_paths: Vec<PathBuf> = entries
                        .filter_map(|entry| entry.ok())
                        .map(|entry| entry.path())
                        .filter(|path| {
                            path.extension()
                                .and_then(|ext| ext.to_str())
                                .map(|ext| matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "gif" | "bmp"))
                                .unwrap_or(false)
                        })
                        .collect();

                    // 发送图片路径到预加载器
                    for path in image_paths {
                        let _ = preloader_clone.send(path);
                    }
                }
            });
        }
    }

    pub fn current_file(&self) -> Option<&PathBuf> {
        self.current_file.as_ref()
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
        // 如果当前文件相同且未在加载中，直接返回
        if self.current_file.as_ref() == Some(&path) && !self.is_loading {
            return;
        }

        // 如果当前正在加载其他文件，设置待处理文件并返回
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
                    // 图片文件预览 - 简化逻辑
                    let mut found = false;

                    // 1. 先检查预加载缓存（最快）
                    if let Some(preloader) = &self.preloader {
                        if let Some((texture, size)) = preloader.get_cached_thumbnail(&path, ctx) {
                            self.image_texture = Some(texture);
                            self.image_size = Some(size);
                            self.preview_content = format!(
                                "图片预览\n\n尺寸: {} x {} 像素\n格式: {}",
                                size.0,
                                size.1,
                                path.extension()
                                    .and_then(|ext| ext.to_str())
                                    .map(|ext| ext.to_uppercase())
                                    .unwrap_or_else(|| "未知".to_string())
                            );
                            self.is_loading = false;
                            found = true;
                        }
                    }

                    // 2. 如果预加载缓存没有，检查普通缓存
                    if !found {
                        if let Some((texture, size)) = self.get_cached_image(&path) {
                            self.image_texture = Some(texture);
                            self.image_size = Some(size);
                            self.preview_content = format!(
                                "图片预览\n\n尺寸: {} x {} 像素\n格式: {}",
                                size.0,
                                size.1,
                                path.extension()
                                    .and_then(|ext| ext.to_str())
                                    .map(|ext| ext.to_uppercase())
                                    .unwrap_or_else(|| "未知".to_string())
                            );
                            self.is_loading = false;
                        } else {
                            // 3. 没有缓存，启动异步加载
                            self.is_loading = true;
                            self.preview_content = "正在加载图片...".to_string();
                            self.start_async_loading(path.clone(), ctx.clone());
                        }
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
        if !self.is_loading || self.loading_result.is_none() {
            // 检查是否有待处理的文件
            if let Some(pending) = self.pending_file.take() {
                self.load_preview(pending, ctx);
            }
            return;
        }

        // 使用简单的检查，避免复杂的借用问题
        let loading_result = self.loading_result.take();
        if let Some(loading_result) = loading_result {
            if let Ok(result_guard) = loading_result.lock() {
                if let Some(result) = result_guard.as_ref() {
                    // 检查结果是否对应当前文件
                    if let Some(current_file) = &self.current_file {
                        if result.file_path == *current_file {
                            let current_file_clone = current_file.clone();
                            if let Some(img_rgba) = &result.img_rgba {
                                if let Some((width, height)) = result.size {
                                    // 从RgbaImage创建ColorImage，避免额外的数据拷贝
                                    let img_size = [img_rgba.width() as usize, img_rgba.height() as usize];
                                    let color_image = egui::ColorImage::from_rgba_premultiplied(img_size, img_rgba);

                                    let texture = ctx.load_texture(
                                        format!("async_image_{}", current_file_clone.display()),
                                        color_image,
                                        egui::TextureOptions::default(),
                                    );

                                    self.image_texture = Some(texture.clone());
                                    self.image_size = Some((width, height));

                                    // 缓存图片以提高后续访问性能
                                    self.cache_image(&current_file_clone, texture, (width, height));

                                    self.preview_content = format!(
                                        "图片预览\n\n尺寸: {} x {} 像素\n格式: {}",
                                        width,
                                        height,
                                        current_file_clone.extension()
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
                            return;
                        }
                    }
                }
            }
            // 如果没有处理结果，重新放回去
            self.loading_result = Some(loading_result);
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
        // 简化缓存键，不包含修改时间以提高性能
        path.to_string_lossy().to_string()
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
        // 保留最近50个图片的缓存，删除其他（增加缓存数量以提高性能）
        if self.texture_cache.len() > 100 {
            let mut keys: Vec<_> = self.texture_cache.keys().cloned().collect();
            keys.sort(); // 简单的字符串排序，实际项目中可能需要更复杂的策略

            for key in keys.iter().take(self.texture_cache.len() - 100) {
                self.texture_cache.remove(key);
            }
        }
    }

    fn get_cached_image(&self, path: &Path) -> Option<(egui::TextureHandle, (u32, u32))> {
        let cache_key = self.get_cache_key(path);
        if let Some(cached) = self.texture_cache.get(&cache_key) {
            // 简化缓存有效性检查，只在文件大小变化时才重新验证
            if let Ok(metadata) = path.metadata() {
                if cached.file_size == metadata.len() {
                    return Some((cached.texture.clone(), cached.size));
                }
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

    // 在后台线程中加载图片 - 简化版本，只生成缩略图
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

        // 检查是否为图片格式
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

        // 直接加载并生成缩略图 (最大800px)
        match image::open(path) {
            Ok(img) => {
                let (width, height) = img.dimensions();

                // 统一生成400px缩略图
                let thumbnail_size = 400;
                let (thumb_width, thumb_height, thumbnail) = if width > thumbnail_size || height > thumbnail_size {
                    let scale = (thumbnail_size as f32 / width.max(height) as f32).min(1.0);
                    let new_width = (width as f32 * scale) as u32;
                    let new_height = (height as f32 * scale) as u32;

                    let thumbnail = img.resize(
                        new_width,
                        new_height,
                        image::imageops::FilterType::Nearest // 使用快速缩放
                    );
                    (new_width, new_height, thumbnail)
                } else {
                    (width, height, img)
                };

                let img_rgba = thumbnail.to_rgba8();

                LoadingResult {
                    img_rgba: Some(img_rgba),
                    size: Some((thumb_width, thumb_height)),
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