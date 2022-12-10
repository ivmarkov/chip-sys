use autocxx::WithinUniquePtr;
use std::pin::Pin;

use connectedhomeip_sys::*;

fn main() {
    foo::foo();
}

//     chkerr!(unsafe { chip::Platform::MemoryInit(core::ptr::null_mut(), 0) }).unwrap();

//     let mut platform_mgr =
//         unsafe { Pin::new_unchecked(singleton_raw::platform_mgr().as_mut().unwrap()) };

//     chkerr!(platform_mgr.as_mut().InitChipStack()).unwrap();

//     println!("Initialized");

//     let mut init_params = chip::CommonCaseDeviceServerInitParams::new().within_unique_ptr();

//     // Init Data Model and CHIP App Server
//     chkerr!(init_params
//         .as_mut()
//         .unwrap()
//         .InitializeStaticResourcesBeforeServerInit())
//     .unwrap();

//     let mut server = unsafe { Pin::new_unchecked(singleton_raw::server().as_mut().unwrap()) };

//     chkerr!(server.as_mut().Init(init_params.as_ref().unwrap().as_ref())).unwrap();

//     platform_mgr.as_mut().RunEventLoop();

//     println!("Exiting");
// }

// // fn run_chip_loop() {

// // }

// #[no_mangle]
// extern "C" fn rustEmberAfActionsClusterInstantActionCallback(
//     commandObj: *mut chip::app::CommandHandler,
//     commandPath: *const chip::app::ConcreteCommandPath,
//     commandData: *const chip::app::Clusters::Actions::Commands::InstantAction::DecodableType,
// ) -> bool {
//     true
// }
