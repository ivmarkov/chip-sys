use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, iter};

use anyhow::Result;

use embuild::build::{CInclArgs, LinkArgs};
use embuild::{cmd, git, git::sdk};
use pkg_config::Library;
use tempfile::NamedTempFile;

const CHIP_PATH: &str = "CHIP_PATH";
const CHIP_REPOSITORY: &str = "CHIP_REPOSITORY";
const CHIP_VERSION: &str = "CHIP_VERSION";

const CHIP_DEFAULT_REPOSITORY: &str = "https://github.com/project-chip/connectedhomeip";
const CHIP_DEFAULT_VERSION: &str = "branch:v1.0-branch";
const CHIP_MANAGED_REPO_DIR_BASE: &str = "repos";

const WORKSPACE_INSTALL_DIR: &str = ".embuild/chip";

// TODO: Parameterize
const LINUX: bool = true;
const BLE: bool = true;

const BLE_LINUX_LIBS: &[(&str, &str)] = &[
    ("glib-2.0", "2.0"),
    ("gobject-2.0", "2.0"),
    ("gio-2.0", "2.0"),
];

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
    "glue::Initialize",
    "glue::CommonCaseDeviceServerInitParams",
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
    let sdk = get_chip()?;

    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?).canonicalize()?;
    let chip_out_dir = out_dir.join("chip");

    let sdk_repo = build_chip(&sdk, &chip_out_dir)?;

    let includes = get_chip_includes(&sdk_repo, &chip_out_dir)?;
    let libs = get_chip_libs(&sdk_repo, &chip_out_dir)?;
    let libp = get_chip_lib_paths(&sdk_repo, &chip_out_dir)?;

    gen_bindings(&includes, &out_dir)?;

    let incl_args = CInclArgs {
        args: includes
            .into_iter()
            .map(|incl| format!("-I{}", incl.display()))
            .collect::<Vec<_>>()
            .join(" "),
    };

    let lib_args = LinkArgs {
        args: libs
            .into_iter()
            .map(|lib| format!("-l{lib}"))
            .chain(libp.into_iter().map(|libp| format!("-L{}", libp.display())))
            .collect::<Vec<_>>(),
    };

    incl_args.propagate();
    lib_args.propagate();
    lib_args.output();

    Ok(())
}

fn build_chip(sdk: &sdk::SdkOrigin, chip_out_dir: &Path) -> Result<git::Repository> {
    let sdk_repo = match sdk {
        sdk::SdkOrigin::Managed(remote) => {
            let sdks_root = PathBuf::from(WORKSPACE_INSTALL_DIR);
            fs::create_dir_all(&sdks_root)?;

            remote.open_or_clone(
                &sdks_root,
                Default::default(),
                CHIP_DEFAULT_REPOSITORY,
                CHIP_MANAGED_REPO_DIR_BASE,
            )?
        }
        sdk::SdkOrigin::Custom(repo) => repo.clone(),
    };

    let sdk = sdk_repo.worktree().canonicalize()?;

    fs::create_dir_all(chip_out_dir)?;

    let lib = PathBuf::from("lib").canonicalize()?;

    let sdkd = sdk.display();
    let libd = lib.display();
    let chip_out_dird = chip_out_dir.display();

    let mut script = NamedTempFile::new()?;

    write!(&mut script, "set -e; export CHIP_PATH={sdkd}; . {sdkd}/scripts/activate.sh; cd {libd}; gn gen {chip_out_dird}; ninja -C {chip_out_dird}; cd ..")?;
    script.flush()?;

    cmd!("bash", script.path()).run()?;

    Ok(sdk_repo)
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

fn get_chip() -> Result<sdk::SdkOrigin> {
    let sdk = if let Ok(sdk) = std::env::var(CHIP_PATH) {
        sdk::SdkOrigin::Custom(git::Repository::new(PathBuf::from(sdk)))
    } else {
        //panic!();

        sdk::SdkOrigin::Managed(git::sdk::RemoteSdk {
            repo_url: std::env::var(CHIP_REPOSITORY).ok(),
            git_ref: git::Ref::parse(
                std::env::var(CHIP_VERSION).unwrap_or(CHIP_DEFAULT_VERSION.to_owned()),
            ),
        })
    };

    Ok(sdk)
}

fn get_chip_includes(sdk: &git::Repository, chip_out_dir: &Path) -> Result<Vec<PathBuf>> {
    let sdk = sdk.worktree().canonicalize()?;

    let third_party = sdk.join("third_party");

    let includes = [
        // Ours
        PathBuf::from("lib"),
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
    .chain(
        get_pkg_libs(false)?
            .into_iter()
            .flat_map(|lib| lib.include_paths.into_iter()),
    )
    .collect::<Vec<_>>();

    Ok(includes)
}

fn get_chip_libs(_sdk: &git::Repository, _chip_out_dir: &Path) -> Result<Vec<String>> {
    let libs = iter::once("CHIPALL".to_owned())
        .chain(iter::once("stdc++".to_owned()))
        .chain(iter::once("crypto".to_owned()))
        .chain(
            get_pkg_libs(false)?
                .into_iter()
                .flat_map(|lib| lib.libs.into_iter()),
        )
        .collect::<Vec<_>>();

    Ok(libs)
}

fn get_chip_lib_paths(_sdk: &git::Repository, chip_out_dir: &Path) -> Result<Vec<PathBuf>> {
    let libp = iter::once(chip_out_dir.to_owned())
        .chain(
            get_pkg_libs(false)?
                .into_iter()
                .flat_map(|lib| lib.link_paths.into_iter()),
        )
        .collect::<Vec<_>>();

    Ok(libp)
}

fn get_pkg_libs(emit_cargo_metadata: bool) -> Result<Vec<Library>> {
    let libs = if LINUX && BLE {
        BLE_LINUX_LIBS
            .iter()
            .map(|(lib, ver)| {
                pkg_config::Config::new()
                    .cargo_metadata(emit_cargo_metadata)
                    .atleast_version(ver)
                    .probe(lib)
                    .unwrap()
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    Ok(libs)
}
