use core::{ffi, fmt, slice, str};

use crate::CHIP_ERROR;

/// A wrapped [`CHIP_ERROR`] to check if an error occurred.
///
/// A [`CHIP_ERROR`] is returned from most APIs as a status code. If it is equal
/// to [0] it means **no** error occurred.
#[derive(Copy, Clone, Debug)]
pub struct ChipError(CHIP_ERROR);

impl ChipError {
    /// Wrap a [CHIP_ERROR], return [`Some`] if `error` is **not** [0].
    pub const fn from(error: CHIP_ERROR) -> Option<Self> {
        if error.mError == 0 {
            None
        } else {
            Some(Self(error))
        }
    }

    /// Convert `error` into a [`Result`] with `Ok(value)` if no error occurred.
    ///
    /// If `error` is [0] return [`Ok`] of `value` otherwise return [`Err`] of
    /// wrapped `error`.
    pub fn check_and_return<T>(error: CHIP_ERROR, value: T) -> Result<T, Self> {
        if error.mError == 0 {
            Ok(value)
        } else {
            Err(Self(error))
        }
    }

    /// Convert `error` into a [`Result`] with `Ok(())` if not error occurred..
    ///
    /// If `error` equals to [0] return [`Ok`], otherwise return [`Err`] with the
    /// wrapped [`CHIP_ERROR`].
    pub fn convert_code(error: u32) -> Result<(), Self> {
        Self::check_and_return(
            CHIP_ERROR {
                mError: error,
                mFile: core::ptr::null(),
                mLine: 0,
            },
            (),
        )
    }

    /// Convert `error` into a [`Result`] with `Ok(())` if not error occurred..
    ///
    /// If `error` equals to [0] return [`Ok`], otherwise return [`Err`] with the
    /// wrapped [`CHIP_ERROR`].
    pub fn convert(error: CHIP_ERROR) -> Result<(), Self> {
        Self::check_and_return(error, ())
    }

    /// Panic with a specific error message of the contained [`CHIP_ERROR`].
    #[track_caller]
    pub fn panic(&self) {
        panic!("CHIP ERROR: {self}");
    }

    /// Get the wrapped [`CHIP_ERROR`].
    pub fn code(&self) -> u32 {
        unsafe { self.0.AsInteger() }
    }

    pub fn to_raw(result: Result<(), ChipError>) -> CHIP_ERROR {
        match result {
            Result::Ok(()) => CHIP_ERROR {
                mError: 0,
                mFile: core::ptr::null(),
                mLine: 0,
            },
            Result::Err(err) => err.0,
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ChipError {}

impl fmt::Display for ChipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe fn strlen(c_s: *const ffi::c_char) -> usize {
            let mut len = 0;
            while *c_s.offset(len) != 0 {
                len += 1;
            }

            len as usize
        }

        unsafe {
            let c_s = self.0.AsString();
            str::from_utf8_unchecked(slice::from_raw_parts(c_s as *const u8, strlen(c_s))).fmt(f)
        }
    }
}

/// Convert a [`CHIP_ERROR`] into a [`Result<(), ChipError>`](Result).
///
/// See [`ChipError::convert`].
#[macro_export]
macro_rules! chip {
    ($err:expr) => {{
        $crate::ChipError::convert($err as $crate::CHIP_ERROR)
    }};
}

/// Convert a [`CHIP_ERROR`] into a [`Result<T, ChipError>`](Result).
///
/// See [`ChipError::check_and_return`].
#[macro_export]
macro_rules! chip_result {
    ($err:expr, $value:expr) => {{
        $crate::ChipError::check_and_return($err as $crate::CHIP_ERROR, $value)
    }};
}

/// Panic with an error-specific message if `err` is not [0].
///
/// See [`ChipError::from`] and [`ChipError::panic`].
#[macro_export]
macro_rules! chip_nofail {
    ($err:expr) => {{
        if let ::core::option::Option::Some(error) =
            $crate::ChipError::from($err as $crate::CHIP_ERROR)
        {
            error.panic();
        }
    }};
}
