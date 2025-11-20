use eframe::egui;
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic;
use std::thread;
use crossbeam_channel::{self, Sender, Receiver};
use crate::utils;
use image::GenericImageView;
use sysinfo::System;

// 缓存的纹理结构
#[derive(Clone)]
struct CachedTexture {
    texture: egui::TextureHandle,
    size: (u32, u32),
}

// 计算基于内存的动态缓存大小
fn calculate_cache_sizes() -> (usize, usize) {
    let mut system = System::new_all();
    system.refresh_memory();

    let total_memory = system.total_memory();
    let available_memory = system.available_memory();

    // 使用可用内存的10%作为缓存预算，但限制在合理范围内
    let cache_budget_bytes = (available_memory as f64 * 0.1) as u64;

    // 估算每张图片的平均大小（300px缩略图约 300*300*4 = 360KB）
    const AVG_IMAGE_SIZE: u64 = 360 * 1024; // 360KB

    // 计算可以缓存的图片数量
    let estimated_image_count = (cache_budget_bytes / AVG_IMAGE_SIZE) as usize;

    // 设置合理的范围：最少50张，最多2000张
    let preload_cache_size = estimated_image_count.clamp(50, 2000);
    let main_cache_size = preload_cache_size / 2; // 主缓存稍小一些

    println!("系统内存: {}MB, 可用: {}MB, 预加载缓存: {}张, 主缓存: {}张",
             total_memory / 1024 / 1024,
             available_memory / 1024 / 1024,
             preload_cache_size,
             main_cache_size);

    (preload_cache_size, main_cache_size)
}

pub struct Preview {
    current_file: Option<PathBuf>,
    current_folder: Option<PathBuf>,  // 添加当前文件夹跟踪
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
    // 多线程预加载 - 直接包含，不再使用Option
    preloader: ThumbnailPreloader,
    // 异步文件夹预览
    folder_preview_sender: Option<Sender<(String, Vec<PathBuf>)>>,
    folder_preview_receiver: Option<Receiver<(String, Vec<PathBuf>)>>,
    // 文件信息通道
    file_info_sender: Option<Sender<FileInfo>>,
    file_info_receiver: Option<Receiver<FileInfo>>,
    // 延迟预加载状态
    preload_pending: bool,
    pending_folder: Option<PathBuf>,
    // 动态缓存大小限制
    max_main_cache_size: usize,
    // 图片流预览状态
    image_stream_scroll: f32,
    image_stream_paths: Vec<PathBuf>,
    selected_image_index: Option<usize>,
    pending_image_load: Option<PathBuf>,
}

struct LoadingResult {
    img_rgba: Option<image::RgbaImage>,
    size: Option<(u32, u32)>,
    error: Option<String>,
    file_path: PathBuf,
    folder_content: Option<String>,
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
    sender: Sender<PathBuf>,
    cache: Arc<Mutex<HashMap<String, (image::RgbaImage, (u32, u32))>>>,
    texture_cache: Arc<Mutex<HashMap<String, CachedTexture>>>,
    threads: Vec<thread::JoinHandle<()>>,
    stop_signal: Arc<atomic::AtomicBool>,
    thread_count: usize,
    max_cache_size: usize,  // 动态缓存大小限制
}

impl ThumbnailPreloader {
    fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded::<PathBuf>();
        let cache = Arc::new(Mutex::new(HashMap::new()));
        let texture_cache = Arc::new(Mutex::new(HashMap::new()));

        // 计算动态缓存大小
        let (preload_cache_size, _) = calculate_cache_sizes();

        // 减少线程数量以降低资源消耗：2-8之间
        let thread_count = std::thread::available_parallelism()
            .map(|n| n.get().clamp(2, 6))
            .unwrap_or(4);

        let mut threads = Vec::new();

        // 创建工作线程 - 每个线程独立处理接收到的消息
        for _thread_id in 0..thread_count {
            let receiver = receiver.clone(); // crossbeam Receiver 可以克隆
            let cache_clone = cache.clone();
            threads.push(thread::spawn(move || {
                let mut processed_count = 0;
                while let Ok(image_path) = receiver.recv() {
                    // 检查缓存是否已存在，避免重复处理
                    let cache_key = image_path.to_string_lossy().to_string();
                    let should_process = if let Ok(cache_guard) = cache_clone.lock() {
                        !cache_guard.contains_key(&cache_key)
                    } else {
                        true // 如果无法获取锁，假设需要处理
                    };

                    if should_process {
                        // 动态缓存大小检查
                        if let Ok(mut cache_guard) = cache_clone.lock() {
                            if cache_guard.len() > preload_cache_size {
                                // 只清理最老的20%，保留大部分缓存
                                let cleanup_count = (preload_cache_size / 5).max(10);
                                let keys_to_remove: Vec<_> = cache_guard.keys()
                                    .take(cleanup_count)
                                    .cloned()
                                    .collect();
                                for key in keys_to_remove {
                                    cache_guard.remove(&key);
                                }
                                println!("预加载缓存清理: 移除{}项，当前缓存大小: {}",
                                         cleanup_count, cache_guard.len());
                            }
                        }

                        if let Ok(thumbnail) = Self::generate_thumbnail(&image_path) {
                            let size = (thumbnail.width(), thumbnail.height());
                            if let Ok(mut cache_guard) = cache_clone.lock() {
                                cache_guard.insert(cache_key, (thumbnail, size));
                            }

                            processed_count += 1;
                            // 每个线程处理30张图片后休息一下，减少CPU占用
                            if processed_count % 30 == 0 {
                                std::thread::sleep(std::time::Duration::from_millis(30));
                            }
                        }
                    }
                }
            }));
        }

        Self {
            sender,
            cache,
            texture_cache,
            threads,
            stop_signal: Arc::new(atomic::AtomicBool::new(false)),
            thread_count,
            max_cache_size: preload_cache_size,
        }
    }

    // 优雅关闭预加载器
    fn shutdown(&mut self) {
        self.stop_signal.store(true, atomic::Ordering::SeqCst);
        // 关闭发送通道，让工作线程自然退出
        drop(self.sender.clone());
        
        // 等待所有线程完成
        for thread in self.threads.drain(..) {
            let _ = thread.join();
        }
    }

    // 文件大小检查现在在工作线程中进行，避免阻塞UI

    fn get_cached_thumbnail(&self, path: &Path, ctx: &egui::Context) -> Option<(egui::TextureHandle, (u32, u32))> {
        let cache_key = path.to_string_lossy().to_string();

        // 检查纹理缓存
        if let Ok(texture_cache_guard) = self.texture_cache.lock() {
            if let Some(cached_texture) = texture_cache_guard.get(&cache_key) {
                return Some((cached_texture.texture.clone(), cached_texture.size));
            }
        }

        // 如果纹理缓存没有，检查预加载缓存
        if let Ok(cache_guard) = self.cache.lock() {
            if let Some((rgba_img, size)) = cache_guard.get(&cache_key) {
                // 在主线程创建纹理
                let color_image = egui::ColorImage::from_rgba_premultiplied(
                    [rgba_img.width() as usize, rgba_img.height() as usize],
                    rgba_img
                );
                let texture = ctx.load_texture(
                    format!("preloaded_{}", cache_key),
                    color_image,
                    egui::TextureOptions::default(),
                );
                
                // 缓存纹理避免重复创建
                if let Ok(mut texture_cache_guard) = self.texture_cache.lock() {
                    texture_cache_guard.insert(cache_key, CachedTexture {
                        texture: texture.clone(),
                        size: *size,
                    });
                }
                
                Some((texture, *size))
            } else {
                None
            }
        } else {
            None
        }
    }

    // 检查图片是否已缓存
    fn is_cached(&self, path: &Path) -> bool {
        let cache_key = path.to_string_lossy().to_string();
        
        // 检查纹理缓存
        if let Ok(texture_cache_guard) = self.texture_cache.lock() {
            if texture_cache_guard.contains_key(&cache_key) {
                return true;
            }
        }
        
        // 检查预加载缓存
        if let Ok(cache_guard) = self.cache.lock() {
            if cache_guard.contains_key(&cache_key) {
                return true;
            }
        }
        
        false
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
        // 创建异步文件夹预览通道
        let (folder_sender, folder_receiver) = crossbeam_channel::unbounded();
        
        // 创建文件信息通道
        let (file_info_sender, file_info_receiver) = crossbeam_channel::unbounded();

        // 计算动态缓存大小
        let (_, main_cache_size) = calculate_cache_sizes();

        Self {
            current_file: None,
            current_folder: None,  // 初始化当前文件夹跟踪
            preview_content: String::new(),
            file_info: FileInfo::default(),
            image_texture: None,
            image_size: None,
            texture_cache: HashMap::new(),
            is_loading: false,
            pending_file: None,
            loading_result: None,
            preloader: ThumbnailPreloader::new(), // 直接初始化预加载器
            folder_preview_sender: Some(folder_sender),
            folder_preview_receiver: Some(folder_receiver),
            file_info_sender: Some(file_info_sender),
            file_info_receiver: Some(file_info_receiver),
            preload_pending: false,
            pending_folder: None,
            max_main_cache_size: main_cache_size,
            // 图片流预览状态初始化
            image_stream_scroll: 0.0,
            image_stream_paths: Vec::new(),
            selected_image_index: None,
            pending_image_load: None,
        }
    }

    // 请求延迟预加载文件夹中的图片
    pub fn request_delayed_preload(&mut self, folder_path: &Path) {
        self.preload_pending = true;
        self.pending_folder = Some(folder_path.to_path_buf());

        // 显示缓存状态信息
        let preload_cache_size = if let Ok(cache_guard) = self.preloader.cache.lock() {
            cache_guard.len()
        } else {
            0
        };

        println!("请求延迟预加载: {:?}, 当前预加载缓存: {}项, 主缓存: {}项",
                folder_path, preload_cache_size, self.texture_cache.len());
    }

    // 初始化预加载器 (已废弃，预加载器现在总是初始化)
    #[allow(dead_code)]
    pub fn init_preloader(&mut self) {
        println!("预加载器已初始化");
    }

    // 预加载文件夹中的所有图片 - 延迟启动版本
    pub fn preload_folder_images(&mut self, folder_path: &Path) {
        // 检查是否是新文件夹，如果是则清理预加载缓存
        if let Some(current_folder) = &self.current_folder {
            if current_folder != folder_path {
                println!("文件夹发生变化，清理预加载缓存");
                self.clear_preloader_cache();
            }
        }

        self.current_folder = Some(folder_path.to_path_buf());
        println!("开始预加载文件夹: {:?}", folder_path);

        let preloader_clone = self.preloader.sender.clone();
        let folder_path = folder_path.to_path_buf();

        // 立即启动预加载，移除延迟
        thread::spawn(move || {
            // 使用更高效的文件遍历方式，避免一次性读取所有文件
            if let Ok(entries) = fs::read_dir(&folder_path) {
                let mut image_count = 0;
                let mut paths = Vec::new();

                // 先收集图片路径，避免在循环中发送
                for entry in entries.flatten() {
                    let path = entry.path();

                    // 快速检查文件扩展名，避免不必要的操作
                    if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
                        let ext_lower = ext.to_lowercase();
                        if matches!(ext_lower.as_str(), "jpg" | "jpeg" | "png" | "gif" | "bmp") {
                            paths.push(path);
                            image_count += 1;
                        }
                    }
                }

                println!("检测到 {} 张图片，立即开始预加载", image_count);

                // 批量发送图片路径，减少通道压力
                for path in paths {
                    let _ = preloader_clone.send(path);
                    // 减少发送频率，避免瞬间大量任务
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }

                println!("预加载任务已全部发送");
            }
        });
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

    // 清理资源，关闭预加载器
    pub fn cleanup(&mut self) {
        self.preloader.shutdown();
        self.texture_cache.clear();
        // 重新初始化预加载器以保持可用性
        self.preloader = ThumbnailPreloader::new();
    }

    // 清理预加载缓存，用于切换文件夹时重置状态
    pub fn clear_preloader_cache(&mut self) {
        // 不清空缓存！预加载的图片应该在全局范围内有效
        // 只需要更新current_folder即可，让新文件夹的预加载继续使用已有缓存
        println!("文件夹切换，保留预加载缓存以供复用");
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
            // 异步生成文件夹预览
            self.generate_folder_preview(&path);
            // 立即开始预加载文件夹中的图片
            self.preload_folder_images(&path);
        } else {
            // 对于文件，也触发预加载其所在文件夹的图片
            if let Some(parent) = path.parent() {
                self.preload_folder_images(parent);
            }
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
                    if let Some((texture, size)) = self.preloader.get_cached_thumbnail(&path, ctx) {
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

        // 异步获取文件信息（避免阻塞UI）
        let path_clone = path.clone();
        let file_info_sender = self.file_info_sender.clone();
        
        std::thread::spawn(move || {
            let mut file_info = FileInfo::default();
            if let Ok(metadata) = fs::metadata(&path_clone) {
                file_info.size = utils::get_file_size_str(metadata.len());
                file_info.modified = utils::get_file_modified_time(&path_clone)
                    .unwrap_or_else(|| "未知时间".to_string());
            }
            file_info.file_type = if path_clone.is_dir() {
                "文件夹".to_string()
            } else {
                path_clone.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.to_uppercase())
                    .unwrap_or_else(|| "文件".to_string())
            };
            
            // 通过通道发送文件信息
            if let Some(sender) = file_info_sender {
                let _ = sender.send(file_info);
            }
        });
        
        // 临时设置基本信息（避免UI卡顿）
        self.file_info.file_type = self.get_file_type(&path);
        self.file_info.size = "计算中...".to_string();
        self.file_info.modified = "计算中...".to_string();
    }

    // 在每帧更新时调用，用于处理异步加载结果和延迟预加载
    pub fn update(&mut self, ctx: &egui::Context) {
        // 首先处理文件夹预览通道
        if let Some(receiver) = &self.folder_preview_receiver {
            while let Ok((preview_content, image_paths)) = receiver.try_recv() {
                self.preview_content = preview_content;
                self.image_stream_paths = image_paths;
            }
        }

        // 处理文件信息通道
        if let Some(receiver) = &self.file_info_receiver {
            while let Ok(file_info) = receiver.try_recv() {
                self.file_info = file_info;
            }
        }

        // 处理图片加载请求
        if let Some(image_path) = self.pending_image_load.take() {
            self.load_preview(image_path, ctx);
        }

        // 处理延迟预加载请求
        if self.preload_pending {
            self.preload_pending = false;
            if let Some(folder_path) = self.pending_folder.take() {
                println!("开始延迟预加载: {:?}", folder_path);
                self.preload_folder_images(&folder_path);
            }
        }

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
                    // 图片预览逻辑已在前面的load_preview方法中处理
                    // 这里不需要重复处理，避免无限递归
                }
                _ => {
                    self.preview_content = "此文件类型不支持预览".to_string();
                }
            }
        }
    }

    fn generate_folder_preview(&mut self, path: &Path) {
        // 显示加载状态，避免UI卡顿
        self.preview_content = "正在加载文件夹内容...".to_string();
        
        // 克隆路径和发送器用于异步操作
        let path = path.to_path_buf();
        let preloader_sender = self.preloader.sender.clone();
        if let Some(sender) = self.folder_preview_sender.clone() {
            
            // 在后台线程中读取文件夹内容
            std::thread::spawn(move || {
                let mut folders = Vec::new();
                let mut files = Vec::new();
                let mut image_paths = Vec::new();
                
                // 在后台线程中执行文件系统操作
                if let Ok(entries) = fs::read_dir(&path) {
                    // 限制最多读取100个条目，避免UI卡顿
                    for entry in entries.flatten().take(100) {
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
                            // 检查是否为图片文件
                            if let Some(ext) = entry_path.extension() {
                                if let Some(ext_str) = ext.to_str() {
                                    let ext_lower = ext_str.to_lowercase();
                                    if matches!(ext_lower.as_str(), "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp") {
                                        image_paths.push(entry_path.clone());
                                        // 立即发送到预加载器，不等待
                                        let _ = preloader_sender.send(entry_path);
                                    }
                                }
                            }
                        }
                    }
                }
                
                // 生成预览内容
                let preview_content = if !folders.is_empty() || !files.is_empty() {
                    let mut content = format!(
                        "文件夹内容 ({} 个文件夹, {} 个文件)\n\n📁 文件夹:\n{}\n\n📄 文件:\n{}",
                        folders.len(),
                        files.len(),
                        folders.iter().take(20).map(|f| format!("  {}", f)).collect::<Vec<_>>().join("\n"),
                        files.iter().take(20).map(|f| format!("  {}", f)).collect::<Vec<_>>().join("\n")
                    );
                    
                    if folders.len() > 20 || files.len() > 20 {
                        content.push_str("\n\n... 还有更多项目");
                    }
                    content
                } else {
                    "文件夹为空或无法读取".to_string()
                };
                
                // 通过通道发送预览内容回主线程
                let _ = sender.send((preview_content, image_paths));
            });
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
        let current_file_clone = self.current_file.clone();
        if let Some(path) = &current_file_clone {
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
                    
                    // 显示图片流预览（如果有图片）
                    if !self.image_stream_paths.is_empty() {
                        ui.separator();
                        ui.heading("图片预览");
                        
                        // 显示加载状态和进度
                        let cached_count = self.image_stream_paths.iter()
                            .filter(|path| self.preloader.is_cached(path))
                            .count();
                        let total_count = self.image_stream_paths.len();
                        
                        if cached_count < total_count {
                            ui.label(format!("正在加载图片: {}/{} 已缓存", cached_count, total_count));
                            // 强制请求重绘，确保加载状态及时更新
                            ui.ctx().request_repaint();
                        }
                        
                        // 竖向图片流 - 限制显示数量避免卡顿
                        let max_images_to_show = 20; // 最多显示20张图片
                        for (index, image_path) in self.image_stream_paths.iter().enumerate().take(max_images_to_show) {
                            // 检查图片是否已缓存
                            if self.preloader.is_cached(image_path) {
                                if let Some((texture, size)) = self.preloader.get_cached_thumbnail(image_path, ui.ctx()) {
                                    let mut image_size = egui::vec2(size.0 as f32, size.1 as f32);
                                    // 限制图片宽度为200px，保持比例
                                    let max_width = 200.0;
                                    if image_size.x > max_width {
                                        let scale = max_width / image_size.x;
                                        image_size *= scale;
                                    }
                                    
                                    if image_size.x > 0.0 && image_size.y > 0.0 {
                                        let response = ui.add(
                                            egui::Image::from_texture(egui::load::SizedTexture::new(
                                                texture.id(),
                                                image_size,
                                            ))
                                        );
                                        
                                        // 点击图片预览
                                        if response.clicked() {
                                            self.selected_image_index = Some(index);
                                            self.current_file = Some(image_path.clone());
                                            self.pending_image_load = Some(image_path.clone());
                                        }
                                        
                                        // 鼠标悬停显示文件名
                                        if response.hovered() {
                                            if let Some(file_name) = image_path.file_name() {
                                                response.on_hover_text(file_name.to_string_lossy());
                                            }
                                        }
                                    }
                                }
                            } else {
                                // 显示占位符和加载状态
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.label("加载中...");
                                    if let Some(file_name) = image_path.file_name() {
                                        ui.label(file_name.to_string_lossy());
                                    }
                                });
                                
                                // 触发异步加载（确保只发送一次）
                                let cache_key = image_path.to_string_lossy().to_string();
                                if let Ok(cache_guard) = self.preloader.cache.lock() {
                                    if !cache_guard.contains_key(&cache_key) {
                                        let _ = self.preloader.sender.send(image_path.clone());
                                    }
                                }
                            }
                        }
                        
                        // 如果图片数量超过限制，显示提示信息
                        if self.image_stream_paths.len() > max_images_to_show {
                            ui.label(format!("还有 {} 张图片...", self.image_stream_paths.len() - max_images_to_show));
                        }
                    }
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
        // 动态主缓存清理策略
        if self.texture_cache.len() > self.max_main_cache_size {
            // 只删除最老的20%，保留大部分缓存以提高性能
            let cleanup_count = (self.max_main_cache_size / 5).max(10);
            let keys_to_remove: Vec<_> = self.texture_cache.keys()
                .take(cleanup_count)
                .cloned()
                .collect();

            for key in keys_to_remove {
                self.texture_cache.remove(&key);
            }

            println!("主缓存清理完成，删除了{}项，当前缓存大小: {} / {}",
                     cleanup_count, self.texture_cache.len(), self.max_main_cache_size);
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
                folder_content: None,
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
                folder_content: None,
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
                    folder_content: None,
                }
            }
            Err(e) => {
                LoadingResult {
                    img_rgba: None,
                    size: None,
                    error: Some(format!("无法加载图片: {}", e)),
                    file_path: path.to_path_buf(),
                    folder_content: None,
                }
            }
        }
    }
}