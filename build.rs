fn main() -> std::io::Result<()> {
    // println!("cargo:rerun-if-changed=protos/krpc.proto");
    // println!("cargo:rerun-if-changed=src/krpc.rs");

    prost_build::compile_protos(&["protos/krpc.proto"], &["protos/"])?;
    Ok(())

    // protoc_rust::Codegen::new()
    //     .out_dir("src/")
    //     .inputs(&["protos/krpc.proto"])
    //     .run()
    //     .expect("Running protoc failed.")
}
