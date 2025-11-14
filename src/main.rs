slint::include_modules!();

use std::path::Path;
use std::fs;

fn main() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    
    let ui_weak = ui.as_weak();
    ui.on_path_changed(move |new_path| {
        let ui = ui_weak.unwrap();
        println!("路径已更改: {}", new_path);
    });
    
    let ui_weak = ui.as_weak();
    ui.on_navigate_up(move || {
        println!("向上导航按钮被点击");
    });
    
    ui.run()
}