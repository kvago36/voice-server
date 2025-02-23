use tonic_build;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .compile_protos(
            &[
                "proto/cloudapi/yandex/cloud/ai/stt/v3/stt_service.proto",
                "proto/cloudapi/yandex/cloud/operation/operation.proto",
            ],
            &["proto/googleapis", "proto/cloudapi"],
        )?;
    Ok(())
}
