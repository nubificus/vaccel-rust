fn main() {
    let protos = vec![
        "protos/agent.proto"
    ];

    protos
        .iter()
        .for_each(|p| println!("cargo:rerun-if-changed={}", &p));

    ttrpc_codegen::Codegen::new()
        .out_dir("src")
        .inputs(&protos)
        .include("protos")
        .rust_protobuf()
        .run()
        .expect("Protocol generation failed.");
}
