fn main() {
    // 添加Windows链接库
    #[cfg(windows)]
    {
        println!("cargo:rustc-link-lib=Advapi32");
    }

    // 原有的tauri-build
    tauri_build::build()
}
