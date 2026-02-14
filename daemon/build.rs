fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "macos" {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let stubs = std::path::PathBuf::from(manifest_dir).join("stubs").join("macos");
        println!("cargo:rustc-link-search=native={}", stubs.display());
        println!("cargo:rustc-link-search=framework={}", stubs.display());
        // Allow unresolved symbols at link time — they resolve at runtime on macOS.
        // This avoids maintaining symbol lists in TBD stubs for cross-compilation.
        println!("cargo:rustc-link-arg=-Wl,-undefined,dynamic_lookup");
    }
}
