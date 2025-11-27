use eframe::egui;
use std::path::Path;
use crate::components::preview::{Preview, CachedImage};

/// 缩略图视图模块 - 作为大图标模式的图片显示增强
/// 复用预览组件的纹理缓存，为图片文件提供缩略图显示
pub struct ThumbnailView {
    /// 对预览组件的引用，用于访问已缓存的纹理
    preview_ref: Option<*const Preview>,
}

impl ThumbnailView {
    /// 创建新的缩略图视图
    pub fn new() -> Self {
        Self {
            preview_ref: None,
        }
    }

    /// 设置预览组件引用
    /// 注意：这里使用原始指针，因为生命周期管理比较复杂
    /// 实际使用时，调用者需要确保预览组件的生命周期比缩略图视图长
    pub fn set_preview_ref(&mut self, preview: &Preview) {
        self.preview_ref = Some(preview as *const Preview);
    }

    /// 检查文件是否为支持的图片格式
    pub fn is_image_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                let ext_lower = ext_str.to_lowercase();
                return matches!(ext_lower.as_str(), "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp");
            }
        }
        false
    }

    /// 绘制缩略图（如果可用）
    /// 返回是否成功绘制了缩略图
    pub fn draw_thumbnail_if_available(
        &self,
        ui: &egui::Ui,
        painter: &egui::Painter,
        center_x: f32,
        center_y: f32,
        size: f32,
        file_path: &Path,
    ) -> bool {
        // 检查是否为图片文件
        if !self.is_image_file(file_path) {
            return false;
        }

        // 安全地访问预览组件
        if let Some(preview_ptr) = self.preview_ref {
            // 安全检查：确保指针有效
            if preview_ptr.is_null() {
                return false;
            }

            // 安全地解引用预览组件
            let preview = unsafe { &*preview_ptr };

            // 尝试从预加载缓存中获取缩略图
            if let Some((texture, texture_size)) = preview.preloader.get_cached_thumbnail(file_path, ui.ctx()) {
                // 计算缩略图显示尺寸，保持宽高比
                let (width, height) = texture_size;
                let scale = (size / width as f32).min(size / height as f32);
                let display_width = width as f32 * scale;
                let display_height = height as f32 * scale;

                // 绘制缩略图
                let rect = egui::Rect::from_center_size(
                    egui::pos2(center_x, center_y),
                    egui::vec2(display_width, display_height),
                );

                painter.image(
                    texture.id(),
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );

                return true;
            }

            // 尝试从主缓存中获取缩略图
            if let Some((texture, texture_size)) = preview.get_cached_image(file_path) {
                // 计算缩略图显示尺寸，保持宽高比
                let (width, height) = texture_size;
                let scale = (size / width as f32).min(size / height as f32);
                let display_width = width as f32 * scale;
                let display_height = height as f32 * scale;

                // 绘制缩略图
                let rect = egui::Rect::from_center_size(
                    egui::pos2(center_x, center_y),
                    egui::vec2(display_width, display_height),
                );

                painter.image(
                    texture.id(),
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );

                return true;
            }
        }

        false // 没有可用的缩略图
    }

    /// 检查缩略图是否已缓存
    pub fn is_thumbnail_cached(&self, file_path: &Path) -> bool {
        // 检查是否为图片文件
        if !self.is_image_file(file_path) {
            return false;
        }

        // 安全地访问预览组件
        if let Some(preview_ptr) = self.preview_ref {
            if preview_ptr.is_null() {
                return false;
            }

            let preview = unsafe { &*preview_ptr };
            return preview.preloader.is_cached(file_path);
        }

        false
    }

    /// 请求预加载缩略图（如果还没有缓存）
    pub fn request_thumbnail_preload(&self, file_path: &Path) {
        // 检查是否为图片文件
        if !self.is_image_file(file_path) {
            return;
        }

        // 安全地访问预览组件
        if let Some(preview_ptr) = self.preview_ref {
            if preview_ptr.is_null() {
                return;
            }

            let preview = unsafe { &*preview_ptr };

            // 如果还没有缓存，发送预加载请求
            if !preview.preloader.is_cached(file_path) {
                let _ = preview.preloader.sender.send(file_path.to_path_buf());
            }
        }
    }
}

impl Default for ThumbnailView {
    fn default() -> Self {
        Self::new()
    }
}

// 安全实现：ThumbnailView 只在单个线程中使用，并且不实现 Send/Sync
unsafe impl Send for ThumbnailView {}
unsafe impl Sync for ThumbnailView {}