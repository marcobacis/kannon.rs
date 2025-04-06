fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .compile_protos(
            &[
                ".proto/kannon/mailer/apiv1/mailerapiv1.proto",
                ".proto/kannon/mailer/types/email.proto",
                ".proto/kannon/mailer/types/send.proto",
            ],
            &[".proto"],
        )?;

    Ok(())
}
