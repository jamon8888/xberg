use std::env;
use std::path::{Path, PathBuf};

fn main() {
    if let Err(e) = run() {
        eprintln!("Build script error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").map_err(|_| "CARGO_MANIFEST_DIR not set".to_string())?;

    let config =
        cbindgen::Config::from_file("cbindgen.toml").map_err(|e| format!("Failed to load cbindgen config: {}", e))?;

    cbindgen::generate_with_config(&crate_dir, config)
        .map_err(|e| format!("Failed to generate C bindings: {}", e))?
        .write_to_file("kreuzberg.h");

    // Generate pkg-config files
    let pc_template = std::fs::read_to_string("kreuzberg-ffi.pc.in")
        .map_err(|e| format!("Failed to read pkg-config template: {}", e))?;

    let version = env::var("CARGO_PKG_VERSION").map_err(|_| "CARGO_PKG_VERSION not set".to_string())?;

    let repo_root = Path::new(&crate_dir).parent().and_then(|p| p.parent()).ok_or_else(|| {
        "CARGO_MANIFEST_DIR did not have expected depth (expected crates/kreuzberg-ffi/...)".to_string()
    })?;

    // Normalize paths to use forward slashes for pkg-config compatibility across all platforms
    let dev_prefix = repo_root.to_string_lossy().replace('\\', "/");

    // Platform-specific private libs - detect both OS and target environment
    // Use CARGO_CFG_TARGET_OS for cross-compilation support and CARGO_CFG_TARGET_ENV for MSVC detection
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "unknown".to_string());
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_else(|_| "gnu".to_string());

    let libs_private = match target_os.as_str() {
        "linux" => "-lpthread -ldl -lm",
        "macos" => "-framework CoreFoundation -framework Security -lpthread",
        "windows" => match target_env.as_str() {
            "msvc" => "-lws2_32 -luserenv -lbcrypt",
            // gnu targets (MinGW, etc.) support GCC-specific flags
            "gnu" => "-lpthread -lws2_32 -luserenv -lbcrypt -static-libgcc -static-libstdc++",
            _ => "-lws2_32 -luserenv -lbcrypt",
        },
        _ => "",
    };

    let out_dir = PathBuf::from(env::var("OUT_DIR").map_err(|_| "OUT_DIR not set".to_string())?);
    let profile_dir = out_dir
        .ancestors()
        .nth(3)
        .ok_or_else(|| "OUT_DIR did not have expected depth (expected target/{debug,release}/build/...)".to_string())?;

    // Development version (for monorepo use) - use actual monorepo paths
    // Normalize path separators for pkg-config compatibility across all platforms
    let dev_libdir = profile_dir.to_string_lossy().replace('\\', "/");
    let dev_includedir = format!("{}/crates/kreuzberg-ffi", dev_prefix);
    let dev_pc = format!(
        r#"prefix={}
exec_prefix=${{prefix}}
libdir={}
includedir={}

Name: kreuzberg-ffi
Description: C FFI bindings for Kreuzberg document intelligence library
Version: {}
URL: https://kreuzberg.dev
Libs: -L${{libdir}} -lkreuzberg_ffi
Libs.private: {}
Cflags: -I${{includedir}}
"#,
        dev_prefix, dev_libdir, dev_includedir, version, libs_private
    );
    std::fs::write("kreuzberg-ffi.pc", dev_pc).map_err(|e| format!("Failed to write development pkg-config: {}", e))?;

    // Installation version (for release artifacts)
    let install_pc = pc_template
        .replace("@PREFIX@", "/usr/local")
        .replace("@VERSION@", &version)
        .replace("@LIBS_PRIVATE@", libs_private);
    std::fs::write("kreuzberg-ffi-install.pc", install_pc)
        .map_err(|e| format!("Failed to write installation pkg-config: {}", e))?;

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-arg=-rpath");
        println!("cargo:rustc-link-arg=@loader_path");

        println!("cargo:rustc-link-arg=-rpath");
        println!("cargo:rustc-link-arg=@executable_path/../target/release");
    }

    println!("cargo:rerun-if-changed=cbindgen.toml");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=kreuzberg-ffi.pc.in");

    Ok(())
}
