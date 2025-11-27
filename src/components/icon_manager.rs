use eframe::egui;
use std::collections::HashMap;

pub struct IconManager {
    folder_icon_32: Option<egui::ColorImage>,
    folder_icon_64: Option<egui::ColorImage>,
    exe_icon_25: Option<egui::ColorImage>,
    exe_icon_50: Option<egui::ColorImage>,
    dll_icon_25: Option<egui::ColorImage>,
    dll_icon_50: Option<egui::ColorImage>,
    txt_icon_25: Option<egui::ColorImage>,
    txt_icon_50: Option<egui::ColorImage>,
    code_icon_25: Option<egui::ColorImage>,
    code_icon_50: Option<egui::ColorImage>,
    unidentified_icon_25: Option<egui::ColorImage>,
    unidentified_icon_50: Option<egui::ColorImage>,
    default_icon_25: Option<egui::ColorImage>,
    default_icon_50: Option<egui::ColorImage>,
    texture_id_folder_32: Option<egui::TextureHandle>,
    texture_id_folder_64: Option<egui::TextureHandle>,
    texture_id_exe_25: Option<egui::TextureHandle>,
    texture_id_exe_50: Option<egui::TextureHandle>,
    texture_id_dll_25: Option<egui::TextureHandle>,
    texture_id_dll_50: Option<egui::TextureHandle>,
    texture_id_txt_25: Option<egui::TextureHandle>,
    texture_id_txt_50: Option<egui::TextureHandle>,
    texture_id_code_25: Option<egui::TextureHandle>,
    texture_id_code_50: Option<egui::TextureHandle>,
    texture_id_unidentified_25: Option<egui::TextureHandle>,
    texture_id_unidentified_50: Option<egui::TextureHandle>,
    texture_id_default_25: Option<egui::TextureHandle>,
    texture_id_default_50: Option<egui::TextureHandle>,
    loaded: bool,
}

impl IconManager {
    pub fn new() -> Self {
        Self {
            folder_icon_32: None,
            folder_icon_64: None,
            exe_icon_25: None,
            exe_icon_50: None,
            dll_icon_25: None,
            dll_icon_50: None,
            txt_icon_25: None,
            txt_icon_50: None,
            code_icon_25: None,
            code_icon_50: None,
            unidentified_icon_25: None,
            unidentified_icon_50: None,
            default_icon_25: None,
            default_icon_50: None,
            texture_id_folder_32: None,
            texture_id_folder_64: None,
            texture_id_exe_25: None,
            texture_id_exe_50: None,
            texture_id_dll_25: None,
            texture_id_dll_50: None,
            texture_id_txt_25: None,
            texture_id_txt_50: None,
            texture_id_code_25: None,
            texture_id_code_50: None,
            texture_id_unidentified_25: None,
            texture_id_unidentified_50: None,
            texture_id_default_25: None,
            texture_id_default_50: None,
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
                let egui_image = egui::ColorImage::from_rgba_premultiplied(size, &rgba_image);
                self.folder_icon_32 = Some(egui_image);
            }
        }

        // 加载64px文件夹图标
        if let Ok(image_data) = std::fs::read("material/png/Folder_icon_02_64.png") {
            if let Ok(image) = image::load_from_memory(&image_data) {
                let rgba_image = image.to_rgba8();
                let size = [rgba_image.width() as usize, rgba_image.height() as usize];
                let egui_image = egui::ColorImage::from_rgba_premultiplied(size, &rgba_image);
                self.folder_icon_64 = Some(egui_image);
            }
        }

        // 加载25px EXE图标
        if let Ok(image_data) = std::fs::read("material/png/Exe_icon_0_25.png") {
            if let Ok(image) = image::load_from_memory(&image_data) {
                let rgba_image = image.to_rgba8();
                let size = [rgba_image.width() as usize, rgba_image.height() as usize];
                let egui_image = egui::ColorImage::from_rgba_premultiplied(size, &rgba_image);
                self.exe_icon_25 = Some(egui_image);
            }
        }

        // 加载50px EXE图标
        if let Ok(image_data) = std::fs::read("material/png/Exe_icon_0_50.png") {
            if let Ok(image) = image::load_from_memory(&image_data) {
                let rgba_image = image.to_rgba8();
                let size = [rgba_image.width() as usize, rgba_image.height() as usize];
                let egui_image = egui::ColorImage::from_rgba_premultiplied(size, &rgba_image);
                self.exe_icon_50 = Some(egui_image);
            }
        }

        // 加载25px DLL图标
        if let Ok(image_data) = std::fs::read("material/png/Dll_icon_0_25.png") {
            if let Ok(image) = image::load_from_memory(&image_data) {
                let rgba_image = image.to_rgba8();
                let size = [rgba_image.width() as usize, rgba_image.height() as usize];
                let egui_image = egui::ColorImage::from_rgba_premultiplied(size, &rgba_image);
                self.dll_icon_25 = Some(egui_image);
            }
        }

        // 加载50px DLL图标
        if let Ok(image_data) = std::fs::read("material/png/Dll_icon_0_50.png") {
            if let Ok(image) = image::load_from_memory(&image_data) {
                let rgba_image = image.to_rgba8();
                let size = [rgba_image.width() as usize, rgba_image.height() as usize];
                let egui_image = egui::ColorImage::from_rgba_premultiplied(size, &rgba_image);
                self.dll_icon_50 = Some(egui_image);
            }
        }

        // 加载25px TXT图标
        if let Ok(image_data) = std::fs::read("material/png/Txt_icon_0_25.png") {
            if let Ok(image) = image::load_from_memory(&image_data) {
                let rgba_image = image.to_rgba8();
                let size = [rgba_image.width() as usize, rgba_image.height() as usize];
                let egui_image = egui::ColorImage::from_rgba_premultiplied(size, &rgba_image);
                self.txt_icon_25 = Some(egui_image);
            }
        }

        // 加载50px TXT图标
        if let Ok(image_data) = std::fs::read("material/png/Txt_icon_0_50.png") {
            if let Ok(image) = image::load_from_memory(&image_data) {
                let rgba_image = image.to_rgba8();
                let size = [rgba_image.width() as usize, rgba_image.height() as usize];
                let egui_image = egui::ColorImage::from_rgba_premultiplied(size, &rgba_image);
                self.txt_icon_50 = Some(egui_image);
            }
        }

        // 加载25px代码文件图标
        if let Ok(image_data) = std::fs::read("material/png/Code_icon_0_25.png") {
            if let Ok(image) = image::load_from_memory(&image_data) {
                let rgba_image = image.to_rgba8();
                let size = [rgba_image.width() as usize, rgba_image.height() as usize];
                let egui_image = egui::ColorImage::from_rgba_premultiplied(size, &rgba_image);
                self.code_icon_25 = Some(egui_image);
            }
        }

        // 加载50px代码文件图标
        if let Ok(image_data) = std::fs::read("material/png/Code_icon_0_50.png") {
            if let Ok(image) = image::load_from_memory(&image_data) {
                let rgba_image = image.to_rgba8();
                let size = [rgba_image.width() as usize, rgba_image.height() as usize];
                let egui_image = egui::ColorImage::from_rgba_premultiplied(size, &rgba_image);
                self.code_icon_50 = Some(egui_image);
            }
        }

        // 加载25px无格式文件图标
        if let Ok(image_data) = std::fs::read("material/png/Unidentified_icon_0_25.png") {
            if let Ok(image) = image::load_from_memory(&image_data) {
                let rgba_image = image.to_rgba8();
                let size = [rgba_image.width() as usize, rgba_image.height() as usize];
                let egui_image = egui::ColorImage::from_rgba_premultiplied(size, &rgba_image);
                self.unidentified_icon_25 = Some(egui_image);
            }
        }

        // 加载50px无格式文件图标
        if let Ok(image_data) = std::fs::read("material/png/Unidentified_icon_0_50.png") {
            if let Ok(image) = image::load_from_memory(&image_data) {
                let rgba_image = image.to_rgba8();
                let size = [rgba_image.width() as usize, rgba_image.height() as usize];
                let egui_image = egui::ColorImage::from_rgba_premultiplied(size, &rgba_image);
                self.unidentified_icon_50 = Some(egui_image);
            }
        }

        // 加载25px默认文件图标
        if let Ok(image_data) = std::fs::read("material/png/default_icon_0_25.png") {
            if let Ok(image) = image::load_from_memory(&image_data) {
                let rgba_image = image.to_rgba8();
                let size = [rgba_image.width() as usize, rgba_image.height() as usize];
                let egui_image = egui::ColorImage::from_rgba_premultiplied(size, &rgba_image);
                self.default_icon_25 = Some(egui_image);
            }
        }

        // 加载50px默认文件图标
        if let Ok(image_data) = std::fs::read("material/png/default_icon_0_50.png") {
            if let Ok(image) = image::load_from_memory(&image_data) {
                let rgba_image = image.to_rgba8();
                let size = [rgba_image.width() as usize, rgba_image.height() as usize];
                let egui_image = egui::ColorImage::from_rgba_premultiplied(size, &rgba_image);
                self.default_icon_50 = Some(egui_image);
            }
        }

        self.loaded = true;
        Ok(())
    }

    pub fn ensure_textures(&mut self, ctx: &egui::Context) {
        if self.texture_id_folder_32.is_none() && self.folder_icon_32.is_some() {
            if let Some(ref image) = self.folder_icon_32 {
                self.texture_id_folder_32 = Some(ctx.load_texture(
                    "folder_icon_32",
                    image.clone(),
                    egui::TextureOptions::default(),
                ));
            }
        }

        if self.texture_id_folder_64.is_none() && self.folder_icon_64.is_some() {
            if let Some(ref image) = self.folder_icon_64 {
                self.texture_id_folder_64 = Some(ctx.load_texture(
                    "folder_icon_64",
                    image.clone(),
                    egui::TextureOptions::default(),
                ));
            }
        }

        if self.texture_id_exe_25.is_none() && self.exe_icon_25.is_some() {
            if let Some(ref image) = self.exe_icon_25 {
                self.texture_id_exe_25 = Some(ctx.load_texture(
                    "exe_icon_25",
                    image.clone(),
                    egui::TextureOptions::default(),
                ));
            }
        }

        if self.texture_id_exe_50.is_none() && self.exe_icon_50.is_some() {
            if let Some(ref image) = self.exe_icon_50 {
                self.texture_id_exe_50 = Some(ctx.load_texture(
                    "exe_icon_50",
                    image.clone(),
                    egui::TextureOptions::default(),
                ));
            }
        }

        if self.texture_id_dll_25.is_none() && self.dll_icon_25.is_some() {
            if let Some(ref image) = self.dll_icon_25 {
                self.texture_id_dll_25 = Some(ctx.load_texture(
                    "dll_icon_25",
                    image.clone(),
                    egui::TextureOptions::default(),
                ));
            }
        }

        if self.texture_id_dll_50.is_none() && self.dll_icon_50.is_some() {
            if let Some(ref image) = self.dll_icon_50 {
                self.texture_id_dll_50 = Some(ctx.load_texture(
                    "dll_icon_50",
                    image.clone(),
                    egui::TextureOptions::default(),
                ));
            }
        }

        if self.texture_id_txt_25.is_none() && self.txt_icon_25.is_some() {
            if let Some(ref image) = self.txt_icon_25 {
                self.texture_id_txt_25 = Some(ctx.load_texture(
                    "txt_icon_25",
                    image.clone(),
                    egui::TextureOptions::default(),
                ));
            }
        }

        if self.texture_id_txt_50.is_none() && self.txt_icon_50.is_some() {
            if let Some(ref image) = self.txt_icon_50 {
                self.texture_id_txt_50 = Some(ctx.load_texture(
                    "txt_icon_50",
                    image.clone(),
                    egui::TextureOptions::default(),
                ));
            }
        }

        if self.texture_id_code_25.is_none() && self.code_icon_25.is_some() {
            if let Some(ref image) = self.code_icon_25 {
                self.texture_id_code_25 = Some(ctx.load_texture(
                    "code_icon_25",
                    image.clone(),
                    egui::TextureOptions::default(),
                ));
            }
        }

        if self.texture_id_code_50.is_none() && self.code_icon_50.is_some() {
            if let Some(ref image) = self.code_icon_50 {
                self.texture_id_code_50 = Some(ctx.load_texture(
                    "code_icon_50",
                    image.clone(),
                    egui::TextureOptions::default(),
                ));
            }
        }

        if self.texture_id_unidentified_25.is_none() && self.unidentified_icon_25.is_some() {
            if let Some(ref image) = self.unidentified_icon_25 {
                self.texture_id_unidentified_25 = Some(ctx.load_texture(
                    "unidentified_icon_25",
                    image.clone(),
                    egui::TextureOptions::default(),
                ));
            }
        }

        if self.texture_id_unidentified_50.is_none() && self.unidentified_icon_50.is_some() {
            if let Some(ref image) = self.unidentified_icon_50 {
                self.texture_id_unidentified_50 = Some(ctx.load_texture(
                    "unidentified_icon_50",
                    image.clone(),
                    egui::TextureOptions::default(),
                ));
            }
        }

        if self.texture_id_default_25.is_none() && self.default_icon_25.is_some() {
            if let Some(ref image) = self.default_icon_25 {
                self.texture_id_default_25 = Some(ctx.load_texture(
                    "default_icon_25",
                    image.clone(),
                    egui::TextureOptions::default(),
                ));
            }
        }

        if self.texture_id_default_50.is_none() && self.default_icon_50.is_some() {
            if let Some(ref image) = self.default_icon_50 {
                self.texture_id_default_50 = Some(ctx.load_texture(
                    "default_icon_50",
                    image.clone(),
                    egui::TextureOptions::default(),
                ));
            }
        }
    }

    pub fn get_folder_texture(&self, size: IconSize) -> Option<&egui::TextureHandle> {
        match size {
            IconSize::Small => self.texture_id_folder_32.as_ref(),
            IconSize::Large => self.texture_id_folder_64.as_ref(),
        }
    }

    pub fn get_exe_texture(&self, size: IconSize) -> Option<&egui::TextureHandle> {
        match size {
            IconSize::Small => self.texture_id_exe_25.as_ref(),
            IconSize::Large => self.texture_id_exe_50.as_ref(),
        }
    }

    pub fn get_dll_texture(&self, size: IconSize) -> Option<&egui::TextureHandle> {
        match size {
            IconSize::Small => self.texture_id_dll_25.as_ref(),
            IconSize::Large => self.texture_id_dll_50.as_ref(),
        }
    }

    pub fn get_txt_texture(&self, size: IconSize) -> Option<&egui::TextureHandle> {
        match size {
            IconSize::Small => self.texture_id_txt_25.as_ref(),
            IconSize::Large => self.texture_id_txt_50.as_ref(),
        }
    }

    pub fn get_code_texture(&self, size: IconSize) -> Option<&egui::TextureHandle> {
        match size {
            IconSize::Small => self.texture_id_code_25.as_ref(),
            IconSize::Large => self.texture_id_code_50.as_ref(),
        }
    }

    pub fn get_unidentified_texture(&self, size: IconSize) -> Option<&egui::TextureHandle> {
        match size {
            IconSize::Small => self.texture_id_unidentified_25.as_ref(),
            IconSize::Large => self.texture_id_unidentified_50.as_ref(),
        }
    }

    pub fn get_default_texture(&self, size: IconSize) -> Option<&egui::TextureHandle> {
        match size {
            IconSize::Small => self.texture_id_default_25.as_ref(),
            IconSize::Large => self.texture_id_default_50.as_ref(),
        }
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded &&
        self.texture_id_folder_32.is_some() &&
        self.texture_id_folder_64.is_some() &&
        self.texture_id_exe_25.is_some() &&
        self.texture_id_exe_50.is_some() &&
        self.texture_id_dll_25.is_some() &&
        self.texture_id_dll_50.is_some() &&
        self.texture_id_txt_25.is_some() &&
        self.texture_id_txt_50.is_some() &&
        self.texture_id_code_25.is_some() &&
        self.texture_id_code_50.is_some() &&
        self.texture_id_unidentified_25.is_some() &&
        self.texture_id_unidentified_50.is_some() &&
        self.texture_id_default_25.is_some() &&
        self.texture_id_default_50.is_some()
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