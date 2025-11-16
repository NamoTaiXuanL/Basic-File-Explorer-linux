use std::path::PathBuf;
use std::process::Command;

// 鼠标双击策略
pub struct MouseDoubleClickStrategy;

impl MouseDoubleClickStrategy {
    pub fn new() -> Self {
        Self
    }
    
    // 处理文件双击事件
    pub fn handle_double_click(&self, file_path: PathBuf) -> bool {
        if file_path.is_dir() {
            // 目录双击由其他逻辑处理
            return false;
        }
        
        // 尝试使用系统默认程序打开文件
        if let Err(e) = self.open_file_with_default_program(&file_path) {
            eprintln!("无法打开文件: {:?}, 错误: {}", file_path, e);
            // 这里可以添加弹出打开方式对话框的逻辑
            return false;
        }
        
        true
    }
    
    // 使用系统默认程序打开文件
    fn open_file_with_default_program(&self, file_path: &PathBuf) -> std::io::Result<()> {
        #[cfg(target_os = "windows")]
        {
            // 转换文件路径为Windows格式，并正确处理包含空格的路径
            let path_str = file_path.to_str().unwrap_or_default();
            if path_str.is_empty() {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "文件路径为空"));
            }

            // 使用rundll32调用shell32.dll打开文件，这是更可靠的方式
            Command::new("rundll32")
                .args(["url.dll,FileProtocolHandler", path_str])
                .spawn()?
                .wait()?;
        }
        
        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg(file_path.to_str().unwrap_or_default())
                .spawn()?
                .wait()?;
        }
        
        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open")
                .arg(file_path.to_str().unwrap_or_default())
                .spawn()?
                .wait()?;
        }
        
        Ok(())
    }
}