use std::path::PathBuf;

fn main() -> miette::Result<()> {
    const CHIP_SDK: &str = "/home/ivan/dev/connectedhomeip";
    const CHIP_SDK_BUILD: &str = "out/host";

    let sdk = PathBuf::from(CHIP_SDK);

    // Libs

    println!(
        "cargo:rustc-link-search={}",
        sdk.join(CHIP_SDK_BUILD).join("lib").display()
    );
    //println!("cargo:rustc-link-lib=CHIP");

    // println!(
    //     "cargo:rustc-link-search={}",
    //     sdk.join(CHIP_SDK_BUILD)
    //         .join("obj/src/app/common/lib")
    //         .display()
    // );

    // println!(
    //     "cargo:rustc-link-search={}",
    //     sdk.join(CHIP_SDK_BUILD)
    //         .join("obj/src/app/server/lib")
    //         .display()
    // );

    // println!(
    //     "cargo:rustc-link-search={}",
    //     sdk.join(CHIP_SDK_BUILD).join("obj/src/app/lib").display()
    // );

    println!(
        "cargo:rustc-link-search={}",
        sdk.join("out/linux-x64-all-clusters-no-ble-asan-clang/obj/third_party/connectedhomeip/src/app/common/lib").display()
    );

    println!(
        "cargo:rustc-link-search={}",
        sdk.join("out/linux-x64-all-clusters-no-ble-asan-clang/obj/third_party/connectedhomeip/src/app/server/lib").display()
    );

    println!(
        "cargo:rustc-link-search={}",
        sdk.join("out/linux-x64-all-clusters-no-ble-asan-clang/obj/third_party/connectedhomeip/src/app/lib").display()
    );

    println!("cargo:rustc-link-search={}", "lib/out/custom");

    //println!("cargo:rustc-link-lib=CHIPAppServer");
    //println!("cargo:rustc-link-lib=CHIPDataModel");
    //println!("cargo:rustc-link-lib=ClusterObjects");
    //println!("cargo:rustc-link-lib=MatterDeviceInfoProviderExample");
    println!("cargo:rustc-link-lib=App");

    // TODO: Linux-specific
    let glib = pkg_config::Config::new()
        .cargo_metadata(true)
        .atleast_version("2.0")
        .probe("glib-2.0")
        .unwrap();
    let gobject = pkg_config::Config::new()
        .cargo_metadata(true)
        .atleast_version("2.0")
        .probe("gobject-2.0")
        .unwrap();
    let gio = pkg_config::Config::new()
        .cargo_metadata(true)
        .atleast_version("2.0")
        .probe("gio-2.0")
        .unwrap();

    // TODO: Linux-specific
    println!("cargo:rustc-link-lib=crypto");
    println!("cargo:rustc-link-lib=asan");

    let third_party = sdk.join("third_party");

    let includes = [
        // Ours
        PathBuf::from("src/include"),
        // Generated
        sdk.join(CHIP_SDK_BUILD).join("gen/include"),
        // Generated ZAP includes
        sdk.join("zzz_generated/app-common"),
        // SDK - Linux standalone (TODO: needs config)
        sdk.join("config/standalone"),
        // SDK
        sdk.join("src/include"),
        sdk.join("src"),
        // Third party
        third_party.join("nlassert/repo/include"),
        third_party.join("nlio/repo/include"),
        third_party.join("inipp/repo/inipp"),
    ]
    .into_iter()
    .chain(glib.include_paths.into_iter())
    .chain(gobject.include_paths.into_iter())
    .chain(gio.include_paths.into_iter())
    .collect::<Vec<_>>();

    let mut builder = autocxx_build::Builder::new("src/lib.rs", &includes).build()?;

    builder
        .flag_if_supported("-std=c++14")
        .flag("-Wno-unused")
        .compile("connectedhomeip-sys");

    println!("cargo:rerun-if-changed=src/lib.rs");

    Ok(())
}
