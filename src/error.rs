use core::{ffi, fmt, slice, str};

use crate::{EmberAfStatus, EmberAfStatus_EMBER_ZCL_STATUS_SUCCESS, CHIP_ERROR};

/// A wrapped [`CHIP_ERROR`] to check if an error occurred.
///
/// A [`CHIP_ERROR`] is returned from most APIs as a status code. If it is equal
/// to [0] it means **no** error occurred.
#[derive(Copy, Clone, Debug)]
pub struct ChipError(CHIP_ERROR);

impl ChipError {
    /// Wrap a [CHIP_ERROR]
    pub const fn from(error: CHIP_ERROR) -> Self {
        Self(error)
    }

    /// Wrap a [CHIP_ERROR] code
    pub const fn from_code(error_code: u32) -> Self {
        Self(CHIP_ERROR {
            mError: error_code,
            mFile: core::ptr::null(),
            mLine: 0,
        })
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
    pub fn convert(error: CHIP_ERROR) -> Result<(), Self> {
        Self::check_and_return(error, ())
    }

    /// Panic with a specific error message of the contained [`CHIP_ERROR`].
    #[track_caller]
    pub fn panic(&self) {
        panic!("CHIP ERROR: {self}");
    }

    /// Get the wrapped [`CHIP_ERROR`].
    pub const fn error(self) -> CHIP_ERROR {
        self.0
    }

    /// Get the wrapped [`CHIP_ERROR`] code.
    pub fn code(&self) -> u32 {
        unsafe { self.0.AsInteger() }
    }

    pub const fn to_raw(result: Result<(), ChipError>) -> CHIP_ERROR {
        let err = match result {
            Result::Ok(()) => Self::from_code(0),
            Result::Err(err) => err,
        };

        err.error()
    }
}

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

#[cfg(feature = "std")]
impl std::error::Error for ChipError {}

unsafe impl Send for ChipError {}
unsafe impl Sync for ChipError {}

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

/// A wrapped [`EmberAfStatus`] to check if an error occurred.
///
/// A [`EmberAfStatus`] is returned from most APIs as a status code. If it is equal
/// to [0] it means **no** error occurred.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct EmberAfError(EmberAfStatus);

impl EmberAfError {
    /// Wrap a [`EmberAfStatus`]
    pub const fn from(error: EmberAfStatus) -> Self {
        Self(error)
    }

    /// Convert `error` into a [`Result`] with `Ok(value)` if no error occurred.
    ///
    /// If `error` is [0] return [`Ok`] of `value` otherwise return [`Err`] of
    /// wrapped `error`.
    pub fn check_and_return<T>(error: EmberAfStatus, value: T) -> Result<T, Self> {
        if error == EmberAfStatus_EMBER_ZCL_STATUS_SUCCESS {
            Ok(value)
        } else {
            Err(Self(error))
        }
    }

    /// Convert `error` into a [`Result`] with `Ok(())` if not error occurred..
    ///
    /// If `error` equals to [0] return [`Ok`], otherwise return [`Err`] with the
    /// wrapped [`EmberAfStatus`].
    pub fn convert(error: EmberAfStatus) -> Result<(), Self> {
        Self::check_and_return(error, ())
    }

    /// Panic with a specific error message of the contained [`CHIP_ERROR`].
    #[track_caller]
    pub fn panic(&self) {
        panic!("EMBER AF ERROR: {self}");
    }

    /// Get the wrapped [`EmberAfStatus`].
    pub const fn code(&self) -> EmberAfStatus {
        self.0
    }

    pub const fn to_raw(result: Result<(), EmberAfError>) -> EmberAfStatus {
        match result {
            Result::Ok(()) => EmberAfStatus_EMBER_ZCL_STATUS_SUCCESS,
            Result::Err(err) => err.0,
        }
    }
}

impl fmt::Display for EmberAfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EMBER AF ERROR: {}", self.0)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for EmberAfError {}

/// Convert a [`EmberAfStatus`] into a [`Result<(), EmberAfError>`](Result).
///
/// See [`EmberAfError::convert`].
#[macro_export]
macro_rules! ember {
    ($err:expr) => {{
        $crate::EmberAfError::convert($err as $crate::EmberAfStatus)
    }};
}

/// Convert a [`EmberAfStatus`] into a [`Result<T, EmberAfError>`](Result).
///
/// See [`EmberAfError::check_and_return`].
#[macro_export]
macro_rules! ember_result {
    ($err:expr, $value:expr) => {{
        $crate::EmberAfError::check_and_return($err as $crate::EmberAfStatus, $value)
    }};
}

/// Panic with an error-specific message if `err` is not [0].
///
/// See [`EmberAfError::from`] and [`EmberAfError::panic`].
#[macro_export]
macro_rules! ember_nofail {
    ($err:expr) => {{
        if let ::core::option::Option::Some(error) =
            $crate::EmberAfError::from($err as $crate::EmberAfStatus)
        {
            error.panic();
        }
    }};
}
