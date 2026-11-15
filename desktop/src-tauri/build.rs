fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rerun-if-changed=src/touch_id.m");
        cc::Build::new()
            .file("src/touch_id.m")
            .flag("-fobjc-arc")
            .flag("-fblocks")
            .compile("touch_id");
        println!("cargo:rustc-link-lib=framework=LocalAuthentication");
    }
    tauri_build::build()
}
