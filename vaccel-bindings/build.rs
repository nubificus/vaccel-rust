extern crate bindgen;

use cmake;
use std::path::PathBuf;

fn main() {
    let clang_arg = match pkg_config::Config::new().probe("vaccel") {
        Ok(mut lib) => String::from(format!(
            "-I{}",
            &mut lib
                .include_paths
                .pop()
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap(),
        )),
        Err(_) => {
            let prefix = cmake::build("vaccelrt");
            // Set the -L search path
            println!("cargo:rustc-link-search=native={}/lib", prefix.display());

            // Re-create bindings if top-level vaccelrt header file changes
            println!("cargo:rerun-if-changed=vaccelrt/src/include/vaccel.h");

            format!("-I{}/include", prefix.display())
        }
    };

    println!("vaccelrt include path: {}", clang_arg);

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
        .clang_arg(clang_arg)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
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
    let out_path = PathBuf::from("src");
    bindings
        .write_to_file(out_path.join("ffi.rs"))
        .expect("Couldn't write bindings!");
}
