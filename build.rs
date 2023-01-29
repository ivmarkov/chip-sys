use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs, iter};

use anyhow::Result;

use embuild::build::{CInclArgs, LinkArgs};
use embuild::cargo::workspace_dir;
use embuild::{cmd, git, git::sdk};
use pkg_config::Library;
use tempfile::NamedTempFile;

#[cfg(not(any(target_os = "linux", target_os = "espidf")))]
compile_error!("Currently, `chip-sys` only builds for Linux and ESP-IDF");

const CHIP_PATH: &str = "CHIP_PATH";
const CHIP_REPOSITORY: &str = "CHIP_REPOSITORY";
const CHIP_VERSION: &str = "CHIP_VERSION";

const CHIP_DEFAULT_REPOSITORY: &str = "https://github.com/project-chip/connectedhomeip";
const CHIP_DEFAULT_VERSION: &str = "branch:v1.0-branch";
const CHIP_MANAGED_REPO_DIR_BASE: &str = "repos";

const WORKSPACE_INSTALL_DIR: &str = ".embuild/chip";

static TYPES: &[&str] = &[
    "chip::ChipError",
    "chip::Span",
    "chip::ByteSpan",
    "chip::MutableByteSpan",
    "chip::DeviceLayer::PlatformManager",
    "chip::DeviceLayer::ConfigurationManager",
    "chip::DeviceLayer::ConfigurationManagerImpl",
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

static VARS: &[&str] = &[
    "chip::k.*",
    "CONFIG_.*",
    "INET_CONFIG_.*",
    "CHIP_.*",
    "ZCL_.*",
    "EmberAfStatus_.*",
    "ATTRIBUTE_MASK_.*",
    "CLUSTER_MASK_.*",
    "FIXED_ENDPOINT_COUNT",
];

static FUNCTIONS: &[&str] = &[
    "glue::Initialize",
    "glue::CommonCaseDeviceServerInitParams",
    "chip::Platform::MemoryInit",
    "chip::Server::GetInstance",
    "chip::DeviceLayer::PlatformMgr",
    "chip::DeviceLayer::ConfigurationMgr",
    "chip::DeviceLayer::ConfigurationManagerImpl",
    "chip::Credentials::SetDeviceAttestationCredentialsProvider",
    "chip::Credentials::Examples::GetExampleDACProvider",
    "PrintOnboardingCodes",
    "emberAfEndpointFromIndex",
    "emberAfFixedEndpointCount",
    "emberAfEndpointEnableDisable",
    "emberAfSetDeviceTypeList",
    "emberAfSetDynamicEndpoint",
    "emberAfClearDynamicEndpoint",
    "MatterReportingAttributeChangeCallback",
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
            let sdks_root = workspace_dir().unwrap().join(WORKSPACE_INSTALL_DIR);
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

    let proj_config_include_dir = chip_out_dir.join("app_config");

    create_app_config(&proj_config_include_dir)?;

    let lib = PathBuf::from("lib").canonicalize()?;

    let sdkd = sdk.display();
    let libd = lib.display();
    let chip_out_dird = chip_out_dir.display();
    let proj_config_include_dird = proj_config_include_dir.display();

    let mut script = NamedTempFile::new()?;

    let arg_debug = env::var("PROFILE")?.eq_ignore_ascii_case("debug");

    #[cfg(target_os = "linux")]
    let arg_standalone = true;
    #[cfg(not(target_os = "linux"))]
    let arg_standalone = false;

    #[cfg(feature = "ble")]
    let arg_ble = true;
    #[cfg(not(feature = "ble"))]
    let arg_ble = false;

    #[cfg(feature = "wifi")]
    let arg_wifi = true;
    #[cfg(not(feature = "wifi"))]
    let arg_wifi = false;

    #[cfg(feature = "thread")]
    let arg_thread = true;
    #[cfg(not(feature = "thread"))]
    let arg_thread = false;

    #[cfg(feature = "ipv4")]
    let arg_ipv4 = true;
    #[cfg(not(feature = "ipv4"))]
    let arg_ipv4 = false;

    #[cfg(feature = "tcp")]
    let arg_tcp = true;
    #[cfg(not(feature = "tcp"))]
    let arg_tcp = false;

    write!(
        &mut script,
        "set -e; \
         export CHIP_PATH={sdkd}; \
         export PROJ_CONFIG_INCLUDE_PATH={proj_config_include_dird}; \
         . {sdkd}/scripts/activate.sh; \
         cd {libd}; \
         gn gen \
            {chip_out_dird} \
            '--args= \
                is_debug={arg_debug} \
                standalone={arg_standalone}
                chip_config_network_layer_ble={arg_ble} \
                chip_enable_wifi={arg_wifi} \
                chip_enable_openthread={arg_thread} \
                chip_inet_config_enable_ipv4={arg_ipv4} \
                chip_inet_config_enable_tcp_endpoint={arg_tcp} \
            '; \
         ninja -C {chip_out_dird}; \
         cd ..",
    )?;
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

    for var in VARS {
        bindgen = bindgen.allowlist_var(var);
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
        // App project config
        PathBuf::from(chip_out_dir).join("app_config"),
        // Generated
        PathBuf::from(chip_out_dir).join("gen/include"),
        // Ours
        PathBuf::from("src/include"),
        PathBuf::from("lib/include"),
        #[cfg(target_os = "linux")]
        sdk.join("config/standalone"),
        #[cfg(target_os = "espidf")]
        sdk.join("config/esp32"),
        // Generated ZAP includes
        sdk.join("zzz_generated/bridge-app"),
        sdk.join("zzz_generated/app-common"),
        // SDK
        sdk.join("src/include"),
        sdk.join("src"),
        // Third party
        third_party.join("nlassert/repo/include"),
        third_party.join("nlio/repo/include"),
        #[cfg(target_os = "linux")]
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

#[allow(unused_variables)]
fn get_pkg_libs(emit_cargo_metadata: bool) -> Result<Vec<Library>> {
    #[cfg(target_os = "linux")]
    let libs = {
        const LINUX_LIBS: &[(&str, &str)] = &[
            ("glib-2.0", "2.0"),
            ("gobject-2.0", "2.0"),
            ("gio-2.0", "2.0"),
        ];

        LINUX_LIBS
            .iter()
            .map(|(lib, ver)| {
                pkg_config::Config::new()
                    .cargo_metadata(emit_cargo_metadata)
                    .atleast_version(ver)
                    .probe(lib)
                    .unwrap()
            })
            .collect::<Vec<_>>()
    };

    #[cfg(not(target_os = "linux"))]
    let libs = Vec::new();

    Ok(libs)
}

fn create_app_config(dir: &Path) -> Result<()> {
    #[allow(unused_mut, unused_assignments)]
    let mut arg_dynamic_endpoint_count = 4;

    #[cfg(feature = "endpoints-8")]
    {
        arg_dynamic_endpoint_count = 8;
    }
    #[cfg(feature = "endpoints-16")]
    {
        arg_dynamic_endpoint_count = 16;
    }
    #[cfg(feature = "endpoints-32")]
    {
        arg_dynamic_endpoint_count = 32;
    }
    #[cfg(feature = "endpoints-64")]
    {
        arg_dynamic_endpoint_count = 64;
    }
    #[cfg(feature = "endpoints-128")]
    {
        arg_dynamic_endpoint_count = 128;
    }
    #[cfg(feature = "endpoints-256")]
    {
        arg_dynamic_endpoint_count = 256;
    }
    #[cfg(feature = "endpoints-512")]
    {
        arg_dynamic_endpoint_count = 512;
    }
    #[cfg(feature = "endpoints-1024")]
    {
        arg_dynamic_endpoint_count = 1024;
    }

    fs::create_dir_all(dir)?;

    let mut file = File::create(dir.join("CHIPProjectAppConfig.h"))?;

    write!(
        &mut file,
        "#pragma once
#define CHIP_DEVICE_CONFIG_DYNAMIC_ENDPOINT_COUNT {arg_dynamic_endpoint_count}
#include <CHIPProjectConfig.h>
"
    )?;

    file.flush()?;

    Ok(())
}
