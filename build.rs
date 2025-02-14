use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=lonesha256.h");
    println!("cargo:rerun-if-changed=src/wrapper.c");

    // Link neccessary system libraries
    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=user32");
        println!("cargo:rustc-link-lib=gdi32");
        println!("cargo:rustc-link-lib=shell32");
    } else if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=X11");
        println!("cargo:rustc-link-lib=Xext");
        println!("cargo:rustc-link-lib=Xrender");
        println!("cargo:rustc-link-lib=Xfixes");
    }

    // Compile C library
    cc::Build::new()
        .file("src/wrapper.c")
        .include(".")
        .compile("luigi");

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    // Write bindings
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("ffi.rs"))
        .expect("Couldn't write bindings!");
}
