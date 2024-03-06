fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/mainchain.proto")?;
    tonic_build::compile_protos("proto/sidechain.proto")?;
    Ok(())
}
