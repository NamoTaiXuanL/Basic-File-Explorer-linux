use eframe::egui;
use std::collections::HashMap;

pub struct IconManager {
    folder_icon_32: Option<egui::ColorImage>,
    folder_icon_64: Option<egui::ColorImage>,
    texture_id_32: Option<egui::TextureHandle>,
    texture_id_64: Option<egui::TextureHandle>,
    loaded: bool,
}

impl IconManager {
    pub fn new() -> Self {
        Self {
            folder_icon_32: None,
            folder_icon_64: None,
            texture_id_32: None,
            texture_id_64: None,
            loaded: false,
        }
    }

    pub fn load_icons(&mut self) -> Result<(), String> {
        if self.loaded {
            return Ok(());
        }

        // 加载32px文件夹图标
        if let Ok(image_data) = std::fs::read("material/png/Folder_icon_02_32.png") {
            if let Ok(image) = image::load_from_memory(&image_data) {
                let rgba_image = image.to_rgba8();
                let size = [rgba_image.width() as usize, rgba_image.height() as usize];
                let egui_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba_image.into_raw());
                self.folder_icon_32 = Some(egui_image);
            }
        }

        // 加载64px文件夹图标
        if let Ok(image_data) = std::fs::read("material/png/Folder_icon_02_64.png") {
            if let Ok(image) = image::load_from_memory(&image_data) {
                let rgba_image = image.to_rgba8();
                let size = [rgba_image.width() as usize, rgba_image.height() as usize];
                let egui_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba_image.into_raw());
                self.folder_icon_64 = Some(egui_image);
            }
        }

        self.loaded = true;
        Ok(())
    }

    pub fn ensure_textures(&mut self, ctx: &egui::Context) {
        if self.texture_id_32.is_none() && self.folder_icon_32.is_some() {
            if let Some(ref image) = self.folder_icon_32 {
                self.texture_id_32 = Some(ctx.load_texture(
                    "folder_icon_32",
                    image.clone(),
                    egui::TextureOptions::default(),
                ));
            }
        }

        if self.texture_id_64.is_none() && self.folder_icon_64.is_some() {
            if let Some(ref image) = self.folder_icon_64 {
                self.texture_id_64 = Some(ctx.load_texture(
                    "folder_icon_64",
                    image.clone(),
                    egui::TextureOptions::default(),
                ));
            }
        }
    }

    pub fn get_folder_texture(&self, size: IconSize) -> Option<&egui::TextureHandle> {
        match size {
            IconSize::Small => self.texture_id_32.as_ref(),
            IconSize::Large => self.texture_id_64.as_ref(),
        }
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded && self.texture_id_32.is_some() && self.texture_id_64.is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IconSize {
    Small, // 32px
    Large, // 64px
}

impl Default for IconManager {
    fn default() -> Self {
        Self::new()
    }
}