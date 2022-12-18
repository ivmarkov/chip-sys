use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use embuild::cmd;
use tempfile::NamedTempFile;

const WORKSPACE_INSTALL_DIR: &str = ".embuild/chip";

static TYPES: &'static [&str] = &[
    "chip::ChipError",
    "chip::Span",
    "chip::ByteSpan",
    "chip::MutableByteSpan",
    "chip::DeviceLayer::PlatformManager",
    "chip::DeviceLayer::ConfigurationManager",
    "chip::DeviceLayer::CommissionableDataProvider",
    "chip::RendezvousInformationFlag",
    "chip::RendezvousInformationFlags",
    "chip::Server",
    "chip::ServerInitParams",
    "chip::CommonCaseDeviceServerInitParams",
    "chip::EndpointId",
    "chip::ClusterId",
    "chip::CommandId",
    "chip::DataVersion",
    "chip::app::CommandHandler",
    "chip::app::ConcreteCommandPath",
    "chip::app::Clusters::Actions::Commands::InstantAction::DecodableType",
    "EmberAfStatus",
    "EmberAfDeviceType",
    "EmberAfEndpointType",
    "EmberAfAttributeMetadata",
    "EmberAfClusterMask",
    "EmberAfGenericClusterFunction",
    "EmberAfCluster",
];

static FUNCTIONS: &'static [&str] = &[
    "chip::Platform::MemoryInit",
    "chip::Server::GetInstance",
    "chip::DeviceLayer::PlatformMgr",
    "chip::DeviceLayer::ConfigurationMgr",
    "chip::Credentials::SetDeviceAttestationCredentialsProvider",
    "chip::Credentials::Examples::GetExampleDACProvider",
    "PrintOnboardingCodes",
    "emberAfEndpointFromIndex",
    "emberAfFixedEndpointCount",
    "emberAfEndpointEnableDisable",
    "emberAfSetDeviceTypeList",
    "emberAfSetDynamicEndpoint",
    "emberAfClearDynamicEndpoint",
];

fn main() -> Result<()> {
    build()
}

fn build() -> Result<()> {
    let sdk = PathBuf::from(
        std::env::var("CHIP_ROOT").context("Failed to find `CHIP_ROOT` environment variable")?,
    );
    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    let chip_out_dir = out_dir.join("chip");

    build_chip(&sdk, &chip_out_dir)?;
    let includes = emit_chip_libs(&sdk, &chip_out_dir)?;

    gen_bindings(&includes, &out_dir)?;

    Ok(())
}

fn build_chip(sdk: &Path, chip_out_dir: &Path) -> Result<()> {
    let lib = PathBuf::from("lib");

    let sdkd = sdk.display();
    let libd = lib.display();
    let chip_out_dird = chip_out_dir.display();

    let mut script = NamedTempFile::new()?;
    write!(&mut script, ". {sdkd}/scripts/activate.sh; cd {libd}; gn gen {chip_out_dird}; ninja -C {chip_out_dird}; cd ..")?;
    script.flush()?;

    cmd!("bash", script.path()).run()?;

    Ok(())
}

fn emit_chip_libs(sdk: &Path, chip_out_dir: &Path) -> Result<Vec<PathBuf>> {
    println!("cargo:rustc-link-search={}", chip_out_dir.display());
    println!("cargo:rustc-link-lib=CHIPALL");
    println!("cargo:rustc-link-lib=stdc++");

    // TODO: Linux-specific
    let glib = pkg_config::Config::new()
        .cargo_metadata(true)
        .atleast_version("2.0")
        .probe("glib-2.0")?;
    let gobject = pkg_config::Config::new()
        .cargo_metadata(true)
        .atleast_version("2.0")
        .probe("gobject-2.0")?;
    let gio = pkg_config::Config::new()
        .cargo_metadata(true)
        .atleast_version("2.0")
        .probe("gio-2.0")?;

    // TODO: Linux-specific
    println!("cargo:rustc-link-lib=crypto");

    let third_party = sdk.join("third_party");

    let includes = [
        // Ours
        PathBuf::from("src/include"),
        // Generated
        PathBuf::from(chip_out_dir).join("gen/include"),
        // Generated ZAP includes
        sdk.join("zzz_generated/app-common"),
        sdk.join("zzz_generated/bridge-app"),
        PathBuf::from("lib/include"),
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

    Ok(includes)
}

fn gen_bindings(includes: &[impl AsRef<Path>], out_dir: &Path) -> Result<()> {
    let header = "src/include/bindings.h";

    let mut bindgen = bindgen::Builder::default()
        .generate_inline_functions(true)
        .use_core()
        .enable_function_attribute_detection()
        //.parse_callbacks(Box::new(BindgenCallbacks))
        .header(header)
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++14")
        //.clang_arg("-Wno-unused")
        .vtable_generation(true)
        .opaque_type("std::.*");

    for typ in TYPES {
        bindgen = bindgen
            .allowlist_type(typ)
            .allowlist_function(format!("{typ}_.*"));
    }

    for function in FUNCTIONS {
        bindgen = bindgen.allowlist_function(function);
    }

    for include in includes {
        bindgen = bindgen.clang_arg(format!("-I{}", include.as_ref().display()));
    }

    let bindings = bindgen.generate()?;

    let bindings_file = out_dir.join("bindings.rs");

    bindings.write_to_file(&bindings_file)?;

    println!("cargo:rerun-if-changed={header}");
    println!(
        "cargo:rustc-env=GENERATED_BINDINGS_FILE={}",
        bindings_file.display()
    );

    Ok(())
}
