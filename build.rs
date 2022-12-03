use std::path::PathBuf;

fn main() -> miette::Result<()> {
    const CHIP_SDK: &str = "/home/ivan/dev/connectedhomeip";

    let sdk = PathBuf::from(CHIP_SDK);
    let third_party = sdk.join("third_party");

    let includes = [
        // Ours
        PathBuf::from("src/include"),
        // SDK - Linux standalone (TODO: needs config)
        sdk.join("config/standalone"),
        // SDK
        sdk.join("src/include"),
        sdk.join("src"),
        // Third party
        third_party.join("nlassert/repo/include"),
        third_party.join("nlio/repo/include"),
        third_party.join("inipp/repo/inipp/inipp"),
        // Generated
        sdk.join("zzz_generated/app-common/"),
    ];

    let mut builder = autocxx_build::Builder::new("src/lib.rs", &includes).build()?;

    builder
        .flag_if_supported("-std=c++14")
        .compile("connectedhomeip-sys");

    println!("cargo:rerun-if-changed=src/lib.rs");

    // Add instructions to link to any C++ libraries you need.

    Ok(())
}
