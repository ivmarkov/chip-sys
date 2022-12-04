use connectedhomeip_sys::*;

fn main() {
    chkerr!(unsafe { chip::Platform::MemoryInit(core::ptr::null_mut(), 0) }).unwrap();

    println!("Test");
}
