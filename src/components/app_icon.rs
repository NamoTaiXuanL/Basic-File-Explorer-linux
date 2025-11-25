//! 应用程序图标模块
//!
//! 负责加载和管理应用程序图标，支持窗口标题栏和任务栏图标显示

use eframe::egui;

/// 加载应用程序图标
///
/// 从ICO文件中加载图标数据并转换为egui所需的IconData格式
///
/// # Returns
///
/// 返回包含图标数据的Option，如果加载失败则返回None
pub fn load_app_icon() -> Option<egui::IconData> {
    // 尝试加载ICO格式的应用程序图标
    let icon_path = "material/png/logo_icon_0_150.ico";

    match std::fs::read(icon_path) {
        Ok(icon_data) => {
            // 使用image库解析ICO文件
            match image::load_from_memory(&icon_data) {
                Ok(img) => {
                    let rgba_image = img.to_rgba8();
                    let (width, height) = rgba_image.dimensions();

                    Some(egui::IconData {
                        rgba: rgba_image.into_raw(),
                        width: width,
                        height: height,
                    })
                }
                Err(e) => {
                    eprintln!("警告: 无法解析图标文件 {}: {}", icon_path, e);
                    None
                }
            }
        }
        Err(e) => {
            eprintln!("警告: 无法读取图标文件 {}: {}", icon_path, e);
            None
        }
    }
}

/// 检查图标文件是否存在
///
/// # Returns
///
/// 返回true如果图标文件存在，否则返回false
pub fn icon_file_exists() -> bool {
    std::path::Path::new("material/png/logo_icon_0_150.ico").exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_file_exists() {
        // 这个测试取决于图标文件是否实际存在
        let exists = icon_file_exists();
        println!("Icon file exists: {}", exists);
    }

    #[test]
    fn test_load_app_icon() {
        // 测试图标加载功能
        let icon_data = load_app_icon();
        if let Some(data) = icon_data {
            println!("Icon loaded successfully: {}x{}", data.width, data.height);
        } else {
            println!("Failed to load icon");
        }
    }
}