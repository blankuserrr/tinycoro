fn main(){
    // Build the C library
    cc::Build::new()
        .file("minicoro.c")
        .include("vendor/minicoro")
        .compile("minicoro");

    // Generate bindings using bindgen
    let bindings = bindgen::Builder::default()
        .header("vendor/minicoro/minicoro.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Make it no_std compatible
        .use_core()
        .ctypes_prefix("::core::ffi")
        // Allow non-camel-case types since this is FFI
        .allowlist_function("mco_.*")
        .allowlist_type("mco_.*")
        .allowlist_var("MCO_.*")
        .generate_comments(false)
        .derive_debug(true)
        .derive_copy(true)
        // Don't derive Eq/PartialEq to avoid function pointer comparison warnings
        .derive_eq(false)
        .derive_partialeq(false)
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}