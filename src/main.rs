use std::pin::Pin;

use autocxx::WithinUniquePtr;

use connectedhomeip_sys::*;

pub fn main() {
    chkerr!(unsafe { chip::Platform::MemoryInit(core::ptr::null_mut(), 0) }).unwrap();

    let mut platform_mgr =
        unsafe { Pin::new_unchecked(singleton_raw::platform_mgr().as_mut().unwrap()) };

    chkerr!(platform_mgr.as_mut().InitChipStack()).unwrap();

    println!("Initialized");

    unsafe {
        glue_InitCommissionableDataProvider();
    }

    unsafe {
        chip::Credentials::SetDeviceAttestationCredentialsProvider(
            chip::Credentials::Examples::GetExampleDACProvider(),
        );
    }

    let mut init_params = chip::CommonCaseDeviceServerInitParams::new().within_unique_ptr();

    // Init Data Model and CHIP App Server
    chkerr!(init_params
        .pin_mut()
        .InitializeStaticResourcesBeforeServerInit())
    .unwrap();

    let mut server = unsafe { Pin::new_unchecked(singleton_raw::server().as_mut().unwrap()) };

    chkerr!(server.as_mut().Init(init_params.as_ref().unwrap().as_ref())).unwrap();

    let mut configuration_mgr =
        unsafe { Pin::new_unchecked(singleton_raw::configuration_mgr().as_mut().unwrap()) };

    configuration_mgr.as_mut().LogDeviceConfig();

    //PrintOnboardingCodes(chip::RendezvousInformationFlag(chip::RendezvousInformationFlag::kBLE));

    singleton_raw::print_onboarding_codes();

    println!("Spin loop");

    platform_mgr.as_mut().RunEventLoop();

    println!("Exiting");
}

#[no_mangle]
extern "C" fn gluecb_emberAfActionsClusterInstantActionCallback(
    _command_obj: *mut chip::app::CommandHandler,
    _command_path: *const chip::app::ConcreteCommandPath,
    _command_data: *const chip::app::Clusters::Actions::Commands::InstantAction::DecodableType,
) -> bool {
    true
}

#[no_mangle]
extern "C" fn gluecb_MatterActionsPluginServerInitCallback() {}

#[no_mangle]
extern "C" fn gluecb_CommissionableDataProvider_GetSetupDiscriminator(
    setup_discriminator: *mut u16,
) -> u16 {
    *unsafe { setup_discriminator.as_mut() }.unwrap() = 3840;

    0
}

#[no_mangle]
extern "C" fn gluecb_CommissionableDataProvider_GetSpake2pIterationCount(
    iteration_count: *mut u32,
) -> u16 {
    *unsafe { iteration_count.as_mut() }.unwrap() = 1000;

    0
}

#[no_mangle]
extern "C" fn gluecb_CommissionableDataProvider_GetSpake2pSalt(
    salt_buf: *mut chip::MutableByteSpan,
) -> u16 {
    static SALT: &'static [u8] = b"SPAKE2P Key Salt";

    //VerifyOrReturnError(saltBuf.size() >= kSpake2p_Max_PBKDF_Salt_Length, CHIP_ERROR_BUFFER_TOO_SMALL);

    unsafe { core::slice::from_raw_parts_mut(glue_MutableByteSpan_data(salt_buf), SALT.len()) }
        .copy_from_slice(SALT);

    unsafe {
        glue_MutableByteSpan_reduce_size(salt_buf, SALT.len());
    }

    0
}

#[no_mangle]
extern "C" fn gluecb_CommissionableDataProvider_GetSpake2pVerifier(
    verifier_buf: *mut chip::MutableByteSpan,
    out_verifier_len: *mut usize,
) -> u16 {
    static VERIFIER: &'static [u8] = &[
        0xB9, 0x61, 0x70, 0xAA, 0xE8, 0x03, 0x34, 0x68, 0x84, 0x72, 0x4F, 0xE9, 0xA3, 0xB2, 0x87,
        0xC3, 0x03, 0x30, 0xC2, 0xA6, 0x60, 0x37, 0x5D, 0x17, 0xBB, 0x20, 0x5A, 0x8C, 0xF1, 0xAE,
        0xCB, 0x35, 0x04, 0x57, 0xF8, 0xAB, 0x79, 0xEE, 0x25, 0x3A, 0xB6, 0xA8, 0xE4, 0x6B, 0xB0,
        0x9E, 0x54, 0x3A, 0xE4, 0x22, 0x73, 0x6D, 0xE5, 0x01, 0xE3, 0xDB, 0x37, 0xD4, 0x41, 0xFE,
        0x34, 0x49, 0x20, 0xD0, 0x95, 0x48, 0xE4, 0xC1, 0x82, 0x40, 0x63, 0x0C, 0x4F, 0xF4, 0x91,
        0x3C, 0x53, 0x51, 0x38, 0x39, 0xB7, 0xC0, 0x7F, 0xCC, 0x06, 0x27, 0xA1, 0xB8, 0x57, 0x3A,
        0x14, 0x9F, 0xCD, 0x1F, 0xA4, 0x66, 0xCF,
    ];
    //static VERIFIER: &'static [u8] = b"uWFwqugDNGiEck/po7KHwwMwwqZgN10XuyBajPGuyzUEV/iree4lOrao5GuwnlQ65CJzbeUB49s31EH+NEkg0JVI5MGCQGMMT/SRPFNRODm3wH/MBiehuFc6FJ/NH6Rmzw==";

    if unsafe { glue_MutableByteSpan_size(verifier_buf) } < VERIFIER.len() {
        panic!("!!!");
    }

    unsafe {
        core::slice::from_raw_parts_mut(glue_MutableByteSpan_data(verifier_buf), VERIFIER.len())
    }
    .copy_from_slice(VERIFIER);

    unsafe {
        glue_MutableByteSpan_reduce_size(verifier_buf, VERIFIER.len());
    }

    *unsafe { out_verifier_len.as_mut() }.unwrap() = VERIFIER.len();

    0
}

#[no_mangle]
extern "C" fn gluecb_CommissionableDataProvider_GetSetupPasscode(setup_passcode: *mut u32) -> u16 {
    *unsafe { setup_passcode.as_mut() }.unwrap() = 20202021;

    0
}

extern "C" {
    fn glue_InitCommissionableDataProvider();

    fn glue_MutableByteSpan_data(span: *mut chip::MutableByteSpan) -> *mut u8;
    fn glue_MutableByteSpan_size(span: *mut chip::MutableByteSpan) -> usize;
    fn glue_MutableByteSpan_reduce_size(span: *mut chip::MutableByteSpan, size: usize);
}
