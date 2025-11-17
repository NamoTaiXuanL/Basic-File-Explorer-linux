use eframe::egui;
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
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

        // 先检查缓存，如果有就直接显示
        let cache_key = self.get_cache_key(&path);
        if self.texture_cache.contains_key(&cache_key) {
            // 有缓存，直接显示
            self.is_loading = false;
        } else {
            // 没有缓存，显示加载状态
            self.is_loading = true;
            self.image_texture = None;
            self.image_size = None;
            self.preview_content = "正在加载图片...".to_string();
        }

        // 获取文件信息
        if let Ok(metadata) = fs::metadata(&path) {
            self.file_info.size = utils::get_file_size_str(metadata.len());
            self.file_info.modified = utils::get_file_modified_time(&path)
                .unwrap_or_else(|| "未知时间".to_string());
        }

        self.file_info.file_type = self.get_file_type(&path);

        // 异步生成预览内容
        self.generate_preview(&path, ctx);
    }

    // 在每帧更新时调用，用于处理延迟加载
    pub fn update(&mut self, ctx: &egui::Context) {
        if self.is_loading {
            if let Some(current_file) = self.current_file.clone() {
                // 这里可以添加更复杂的加载逻辑
                // 目前为了简化，直接同步加载但加上状态管理
                self.generate_preview(&current_file, ctx);
                self.is_loading = false;

                // 检查是否有待处理的文件
                if let Some(pending) = self.pending_file.take() {
                    self.load_preview(pending, ctx);
                }
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
                    self.generate_image_preview(path, ctx);
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

    fn generate_image_preview(&mut self, path: &Path, ctx: &egui::Context) {
        // 首先检查缓存
        if let Some((texture, size)) = self.get_cached_image(path) {
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
            return;
        }

        // 首先检查文件是否存在
        if !path.exists() {
            self.preview_content = "文件不存在".to_string();
            self.image_texture = None;
            self.image_size = None;
            return;
        }

        // 检查文件大小，避免加载过大的图片
        if let Ok(metadata) = fs::metadata(path) {
            let file_size_bytes = metadata.len();
            // 限制图片大小为50MB
            if file_size_bytes > 50 * 1024 * 1024 {
                self.preview_content = format!("图片文件过大 ({} MB)，无法预览", file_size_bytes / (1024 * 1024));
                self.image_texture = None;
                self.image_size = None;
                return;
            }
        }

        // 尝试加载图片
        match image::open(path) {
            Ok(img) => {
                let (width, height) = img.dimensions();
                self.image_size = Some((width, height));

                // 检查图片尺寸是否过大
                if width > 8192 || height > 8192 {
                    self.preview_content = format!("图片尺寸过大 ({} x {})，无法预览", width, height);
                    self.image_texture = None;
                    self.image_size = None;
                    return;
                }

                // 将图片转换为RGBA格式
                let img_rgba = img.to_rgba8();
                let size = [img_rgba.width() as usize, img_rgba.height() as usize];

                // 检查图片数据大小
                let expected_size = size[0] * size[1] * 4; // RGBA = 4 bytes per pixel
                if expected_size > 100 * 1024 * 1024 { // 100MB limit for pixel data
                    self.preview_content = format!("图片数据量过大，无法预览");
                    self.image_texture = None;
                    self.image_size = None;
                    return;
                }

                // 创建颜色图像
                let image_data = egui::ColorImage::from_rgba_unmultiplied(size, &img_rgba);

                // 加载纹理
                let texture = ctx.load_texture(
                    format!("cached_image_{}", path.display()),
                    image_data,
                    egui::TextureOptions::default(),
                );

                self.image_texture = Some(texture.clone());

                // 缓存图片
                self.cache_image(path, texture, (width, height));

                self.preview_content = format!(
                    "图片预览\n\n尺寸: {} x {} 像素\n格式: {}\n色彩模式: {:?}",
                    width,
                    height,
                    path.extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext.to_uppercase())
                        .unwrap_or_else(|| "未知".to_string()),
                    img.color()
                );
            }
            Err(e) => {
                self.preview_content = format!("无法加载图片: {}\n请检查文件是否损坏", e);
                self.image_texture = None;
                self.image_size = None;
            }
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
}