#![cfg_attr(not(feature = "std"), no_std)]

pub use bindings::*;
pub use error::*;

pub mod callbacks;
pub mod dynamic;
mod error;

#[allow(clippy::all)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(rustdoc::all)]
#[allow(improper_ctypes)] // TODO: For now, as 5.0 spits out tons of these
mod bindings {
    include!(env!("GENERATED_BINDINGS_FILE"));
}
