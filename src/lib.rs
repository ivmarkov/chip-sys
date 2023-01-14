#![cfg_attr(not(feature = "std"), no_std)]

pub use bindings::*;
pub use error::*;

pub mod cb;
pub mod chip;
mod error;

#[allow(clippy::all)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(rustdoc::all)]
#[allow(improper_ctypes)] // TODO: For now, as 5.0 spits out tons of these
#[allow(clashing_extern_declarations)] // TODO
mod bindings {
    include!(env!("GENERATED_BINDINGS_FILE"));
}
