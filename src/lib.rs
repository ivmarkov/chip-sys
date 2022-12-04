// TODO: autocxx is not compatible yet #![no_std]

use core::fmt::Display;

use autocxx::prelude::*; // use all the main autocxx functions

include_cpp! {
    #include "build-config.h"

    #include "lib/core/CHIPError.h"
    #include "platform/PlatformManager.h"

    safety!(unsafe)
    generate!("chip::ChipError")
    generate!("chip::Platform::MemoryInit")
    generate!("chip::DeviceLayer::PlatformManager")
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
