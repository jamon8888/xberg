fn main() {
    // Set @rpath-relative install_name and add rpath on macOS so the compiled
    // R extension (.so) can locate transitively-linked dylibs like libonnxruntime.dylib.
    // When R compiles the static lib into its extension module, the resulting binary
    // references @rpath/libonnxruntime.dylib via ort-bundled, but without an LC_RPATH
    // entry, dyld cannot resolve it and loading fails with "undefined symbol: OrtGetApiBase".
    //
    // The @loader_path rpath tells dyld to look in the same directory as the .so file,
    // which is where R packages typically place runtime dependencies alongside the
    // extension module.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");
    }

    // On Linux, add $ORIGIN rpath so the compiled R extension (.so) can locate
    // transitively-linked dylibs like libonnxruntime.so at runtime. Without this,
    // R's library.dynam2 dlopen fails with "undefined symbol: OrtGetApiBase".
    // $ORIGIN is the Linux equivalent of @loader_path — it resolves to the
    // directory containing the .so file, where R packages place bundled native
    // dependencies alongside the extension module.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
    }
}
