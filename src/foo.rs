use autocxx::WithinUniquePtr;
use std::pin::Pin;

use crate::*;

pub fn foo() {
    chkerr!(unsafe { chip::Platform::MemoryInit(core::ptr::null_mut(), 0) }).unwrap();

    let mut platform_mgr =
        unsafe { Pin::new_unchecked(singleton_raw::platform_mgr().as_mut().unwrap()) };

    chkerr!(platform_mgr.as_mut().InitChipStack()).unwrap();

    println!("Initialized");

    // if (options.discriminator.HasValue())
    // {
    //     options.payload.discriminator.SetLongValue(options.discriminator.Value());
    // }
    // else
    // {
    //     uint16_t defaultTestDiscriminator = 0;
    //     chip::DeviceLayer::TestOnlyCommissionableDataProvider TestOnlyCommissionableDataProvider;
    //     VerifyOrDie(TestOnlyCommissionableDataProvider.GetSetupDiscriminator(defaultTestDiscriminator) == CHIP_NO_ERROR);

    //     ChipLogError(Support,
    //                  "*** WARNING: Using temporary test discriminator %u due to --discriminator not "
    //                  "given on command line. This is temporary and will disappear. Please update your scripts "
    //                  "to explicitly configure discriminator. ***",
    //                  static_cast<unsigned>(defaultTestDiscriminator));
    //     options.payload.discriminator.SetLongValue(defaultTestDiscriminator);
    // }

    //let cdp = LinuxCommissionableDataProvider::new().within_unique_ptr();


    // cdp.pin_mut().Init(
    //     options.spake2pVerifier, 
    //     options.spake2pSalt, 
    //     spake2pIterationCount, 
    //     setupPasscode,
    //     options.payload.discriminator.GetLongValue());

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

// fn run_chip_loop() {

// }

#[no_mangle]
extern "C" fn rustEmberAfActionsClusterInstantActionCallback(
    commandObj: *mut chip::app::CommandHandler,
    commandPath: *const chip::app::ConcreteCommandPath,
    commandData: *const chip::app::Clusters::Actions::Commands::InstantAction::DecodableType,
) -> bool {
    true
}

#[no_mangle]
extern "C" fn rustMatterActionsPluginServerInitCallback() {
}
