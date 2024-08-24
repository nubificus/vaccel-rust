// SPDX-License-Identifier: Apache-2.0

extern crate cbindgen;

use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("Could not read CARGO_MANIFEST_DIR");
    let target_dir = env::var("CARGO_TARGET_DIR").expect("Could not read CARGO_TARGET_DIR");
    let profile = env::var("PROFILE").expect("Could not read PROFILE");

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_header(format!("// SPDX-License-Identifier: Apache-2.0\n\n// This file is generated by cbindgen {}. Do not edit", cbindgen::VERSION))
        .with_language(cbindgen::Language::C)
        .with_cpp_compat(true)
        .with_line_length(80)
        .with_tab_width(4)
        .with_pragma_once(true)
        .with_no_includes()
        .with_sys_include("stdint.h")
        .with_sys_include("vaccel.h")
        .with_style(cbindgen::Style::Tag)
        .exclude_item("VsockClient")
        .with_after_include("\nstruct VsockClient;")
        // We need nightly to expand macros and parse ffi correctly, so set the
        // 'struct' keyword manually for the affected types for now.
        .rename_item("VsockClient", "struct VsockClient")
        .rename_item("vaccel_arg", "struct vaccel_arg")
        .rename_item("vaccel_tf_buffer", "struct vaccel_tf_buffer")
        .rename_item("vaccel_tf_node", "struct vaccel_tf_node")
        .rename_item("vaccel_tf_tensor", "struct vaccel_tf_tensor")
        .rename_item("vaccel_tflite_tensor", "struct vaccel_tflite_tensor")
        .rename_item("vaccel_torch_buffer", "struct vaccel_torch_buffer")
        .rename_item("vaccel_torch_tensor", "struct vaccel_torch_tensor")
        .rename_item("vaccel_prof_region", "struct vaccel_prof_region")
        .rename_item("vaccel_resource", "struct vaccel_resource")
        .generate()
        .expect("Unable to generate vaccel-client.h")
        .write_to_file(format!("{}/{}/vaccel-client.h", target_dir, profile));
}
