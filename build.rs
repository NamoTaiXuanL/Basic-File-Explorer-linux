#[cfg(target_os = "windows")]
fn main() {
    // 嵌入Windows资源文件
    embed_resource::compile("resource.rc", embed_resource::NONE);
    
    // 设置Windows应用程序子系统为GUI（不显示控制台窗口）
    println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
    println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");
}

#[cfg(not(target_os = "windows"))]
fn main() {
    // 非Windows平台不需要嵌入资源
}