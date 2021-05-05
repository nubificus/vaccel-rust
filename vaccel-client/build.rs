extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let include_path_env = env::var("INCLUDE_PATH").unwrap_or("/usr/local/include".to_string());

    let include_path: Vec<&str> = include_path_env.split(":").collect();

    /*
    let mut clang_arg = String::new();
    for path in include_path {
        clang_arg.push_str(&format!("-I{} ", path));
    }

    println!("Include path: {}", clang_arg);
    */
    /*
    cc::Build::new()
        .file("src/glue.c")
        .include(include_path)
        .compile("vaccel_glue");
    */

    // Rebuild whenever wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    let mut builder = bindgen::Builder::default().header("wrapper.h");

    for path in include_path {
        builder = builder.clang_arg(format!("-I{}", path));
    }
    //.clang_arg(clang_arg.clone())
    let bindings = builder
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .derive_default(true)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
