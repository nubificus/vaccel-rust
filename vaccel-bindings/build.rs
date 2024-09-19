// SPDX-License-Identifier: Apache-2.0

extern crate bindgen;

use std::{env, path::PathBuf};

fn main() {
    let mut lib = pkg_config::Config::new()
        .probe("vaccel")
        .expect("Could not find vaccel");
    let clang_flags: Vec<String> = lib
        .include_paths
        .into_iter()
        .map(|x| format!("-I{}", x.display().to_string()))
        .collect();

    println!("vaccel include path: {}", clang_flags.join(" "));

    // Tell cargo to tell rustc to link to libvaccel.
    println!("cargo:rustc-link-lib=vaccel");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        .clang_args(clang_flags)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Derive Default Trait
        .derive_default(true)
        // Do not prepend C enum name
        .prepend_enum_name(false)
        // FIXME: determine if this causes issues
        // temp add to build on armv7
        .size_t_is_usize(true)
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").expect("Could not read OUT_DIR"));
    bindings
        .write_to_file(out_path.join("ffi.rs"))
        .expect("Couldn't write bindings!");
}
