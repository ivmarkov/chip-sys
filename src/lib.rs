// TODO: autocxx is not no_std compatible yet #![no_std]

use core::fmt::Display;

use autocxx::prelude::*; // use all the main autocxx functions

include_cpp! {
    #include "build-config.h"
    #include "singleton.h"

    #include "lib/core/CHIPError.h"
    #include "platform/PlatformManager.h"
    #include "app/InteractionModelEngine.h"
    #include "app/server/Dnssd.h"
    #include "app/server/Server.h"

    safety!(unsafe)

    generate!("chip::app::CommandHandler")
    generate!("chip::app::ConcreteCommandPath")
    generate!("chip::app::Clusters::Actions::Commands::InstantAction::DecodableType")
    generate!("chip::ChipError")
    generate!("chip::Platform::MemoryInit")
    generate!("chip::DeviceLayer::PlatformManager")
    generate!("chip::Server")
    generate!("chip::ServerInitParams")
    generate!("chip::CommonCaseDeviceServerInitParams")

    generate!("singleton_raw::server")
    generate!("singleton_raw::platform_mgr")
}

pub use ffi::*;

#[derive(Debug)]
pub struct ChipError(u32);

impl ChipError {
    pub const fn new(code: u32) -> Self {
        Self(code)
    }

    pub fn code(&self) -> u32 {
        self.0
    }
}

impl Display for ChipError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CHIP ERROR: {}", self.0)
    }
}

//impl Error for ChipError {}

#[macro_export]
macro_rules! chkerr {
    ($err: expr) => {{
        moveit::moveit! {
            let mut err = $err;
        }

        let code = err.as_mut().AsInteger();

        if code == 0 {
            Ok(())
        } else {
            Err(ChipError::new(code))
        }
    }};
}

// pub mod singleton {
//     pub fn platform_mgr() -> &'static mut chip::DeviceLayer::
// }

pub mod foo;
