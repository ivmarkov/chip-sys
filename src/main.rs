use std::pin::Pin;

use connectedhomeip_sys::*;

fn main() {
    chkerr!(unsafe { chip::Platform::MemoryInit(core::ptr::null_mut(), 0) }).unwrap();

    let mut platform_mgr =
        unsafe { Pin::new_unchecked(singleton_raw::platform_mgr().as_mut().unwrap()) };

    chkerr!(platform_mgr.as_mut().InitChipStack()).unwrap();

    println!("Initialized");

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
