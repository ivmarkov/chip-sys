use std::pin::Pin;

use connectedhomeip_sys::*;

fn main() {
    chkerr!(unsafe { chip::Platform::MemoryInit(core::ptr::null_mut(), 0) }).unwrap();

    chkerr!(unsafe {
        Pin::new_unchecked(singleton_raw::platform_mgr().as_mut().unwrap())
            .as_mut()
            .InitChipStack()
    })
    .unwrap();
}
