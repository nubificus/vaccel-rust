use ttrpc_codegen::Codegen;
use ttrpc_codegen::{Customize, ProtobufCustomize};

fn main() {
    let protos = vec![
        "protos/agent.proto",
        "protos/session.proto",
        "protos/resources.proto",
        "protos/image.proto",
    ];

    protos
        .iter()
        .for_each(|p| println!("cargo:rerun-if-changed={}", &p));

    Codegen::new()
        .out_dir("src")
        .inputs(&protos)
        .include("protos")
        .rust_protobuf()
        .customize(Customize {
            ..Default::default()
        })
        .rust_protobuf_customize(ProtobufCustomize {
            ..Default::default()
        })
        .run()
        .expect("Protocol generation failed.");
}
