extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    // Location of the Tibco dependencies are defined by the TIBRV
    // environment variable.
    let tibrv = env::var("TIBRV").expect("TIBRV not defined.");

    let lib = if env::var("TARGET").unwrap().contains("64") { "tibrv64" } else { "tibrv" };
    println!("cargo:rustc-link-lib={}", lib);
    println!("cargo:rustc-link-search=native={}/lib/", tibrv);

    let bindings = bindgen::Builder::default()
        .rust_target(bindgen::RustTarget::Stable_1_19)
        .header("wrapper.h")
        .clang_arg(format!("-I{}/include/tibrv", tibrv))
        .generate()
        .expect("Unable to generate bindings.");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings.write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings.");
}
