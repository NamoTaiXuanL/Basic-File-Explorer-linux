#[cfg(target_os = "windows")]
fn main() {
    // 嵌入Windows资源文件
    embed_resource::compile("resource.rc", embed_resource::NONE);
}

#[cfg(not(target_os = "windows"))]
fn main() {
    // 非Windows平台不需要嵌入资源
}