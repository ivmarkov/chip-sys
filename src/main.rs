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

    let mut init_params = chip::CommonCaseDeviceServerInitParams::new().within_unique_ptr();

    // Init Data Model and CHIP App Server
    chkerr!(init_params
        .pin_mut()
        .InitializeStaticResourcesBeforeServerInit())
    .unwrap();

    let mut server = unsafe { Pin::new_unchecked(singleton_raw::server().as_mut().unwrap()) };

    chkerr!(server.as_mut().Init(init_params.as_ref().unwrap().as_ref())).unwrap();

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
    _setup_discriminator: *mut u16,
) -> u16 {
    0x02d
}

#[no_mangle]
extern "C" fn gluecb_CommissionableDataProvider_GetSpake2pIterationCount(
    _iteration_count: *mut u32,
) -> u16 {
    0x02d
}

#[no_mangle]
extern "C" fn gluecb_CommissionableDataProvider_GetSpake2pSalt(
    _salt_buf: *mut chip::MutableByteSpan,
) -> u16 {
    0x02d
}

#[no_mangle]
extern "C" fn gluecb_CommissionableDataProvider_GetSpake2pVerifier(
    _verifier_buf: *mut chip::MutableByteSpan,
    _out_verifier_len: *mut usize,
) -> u16 {
    0x02d
}

#[no_mangle]
extern "C" fn gluecb_CommissionableDataProvider_GetSetupPasscode(_setup_passcode: *mut u32) -> u16 {
    0x02d
}

extern "C" {
    fn glue_InitCommissionableDataProvider();
}
