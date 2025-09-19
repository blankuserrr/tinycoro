//! Rust bindings for the minicoro library
//!
//! This crate provides safe and unsafe bindings to minicoro, a minimal asymmetric
//! stackful cross-platform coroutine library in pure C.

#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Include the generated bindings in a private module
mod ffi {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use core::ptr;
use thiserror::Error;

// Re-export only what we need for the public API
#[doc(hidden)]
pub use ffi::mco_coro;

/// A safe wrapper around a minicoro coroutine
pub struct Coroutine {
    inner: *mut mco_coro,
}

impl Coroutine {
    /// Create a new coroutine with the given function and stack size
    ///
    /// # Safety
    ///
    /// The function pointer must be valid and safe to call
    ///
    /// # Errors
    ///
    /// Returns `CoroutineError` if coroutine creation fails
    pub unsafe fn new(
        func: unsafe extern "C" fn(*mut ffi::mco_coro),
        stack_size: usize,
    ) -> Result<Self, CoroutineError> {
        let desc = unsafe { ffi::mco_desc_init(Some(func), stack_size) };
        let mut co: *mut ffi::mco_coro = ptr::null_mut();

        let result = unsafe { ffi::mco_create(&raw mut co, (&raw const desc).cast_mut()) };
        if result == ffi::mco_result_MCO_SUCCESS {
            Ok(Coroutine { inner: co })
        } else {
            Err(CoroutineError::from_raw(result))
        }
    }

    /// Resume the coroutine
    ///
    /// # Errors
    ///
    /// Returns `CoroutineError` if resuming the coroutine fails
    pub fn resume(&mut self) -> Result<(), CoroutineError> {
        let result = unsafe { ffi::mco_resume(self.inner) };
        if result == ffi::mco_result_MCO_SUCCESS {
            Ok(())
        } else {
            Err(CoroutineError::from_raw(result))
        }
    }

    /// Yield the coroutine (call from within coroutine)
    ///
    /// # Errors
    ///
    /// Returns `CoroutineError` if yielding the coroutine fails
    pub fn yield_now(&mut self) -> Result<(), CoroutineError> {
        let result = unsafe { ffi::mco_yield(self.inner) };
        if result == ffi::mco_result_MCO_SUCCESS {
            Ok(())
        } else {
            Err(CoroutineError::from_raw(result))
        }
    }

    /// Get the status of the coroutine
    #[must_use]
    pub fn status(&self) -> CoroutineState {
        let state = unsafe { ffi::mco_status(self.inner) };
        CoroutineState::from_raw(state)
    }

    /// Push data to the coroutine storage
    ///
    /// # Errors
    ///
    /// Returns `CoroutineError` if pushing data fails
    pub fn push<T>(&mut self, data: &T) -> Result<(), CoroutineError> {
        let result = unsafe {
            ffi::mco_push(
                self.inner,
                core::ptr::from_ref::<T>(data).cast::<core::ffi::c_void>(),
                core::mem::size_of::<T>(),
            )
        };
        if result == ffi::mco_result_MCO_SUCCESS {
            Ok(())
        } else {
            Err(CoroutineError::from_raw(result))
        }
    }

    /// Pop data from the coroutine storage
    ///
    /// # Errors
    ///
    /// Returns `CoroutineError` if popping data fails
    pub fn pop<T>(&mut self) -> Result<T, CoroutineError> {
        let mut data = core::mem::MaybeUninit::<T>::uninit();
        let result = unsafe {
            ffi::mco_pop(
                self.inner,
                data.as_mut_ptr().cast::<core::ffi::c_void>(),
                core::mem::size_of::<T>(),
            )
        };
        if result == ffi::mco_result_MCO_SUCCESS {
            Ok(unsafe { data.assume_init() })
        } else {
            Err(CoroutineError::from_raw(result))
        }
    }

    /// Get the number of bytes stored in the coroutine storage
    #[must_use]
    pub fn bytes_stored(&self) -> usize {
        unsafe { ffi::mco_get_bytes_stored(self.inner) }
    }

    /// Get the total storage size
    #[must_use]
    pub fn storage_size(&self) -> usize {
        unsafe { ffi::mco_get_storage_size(self.inner) }
    }
}

impl Drop for Coroutine {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                ffi::mco_destroy(self.inner);
            }
        }
    }
}

unsafe impl Send for Coroutine {}

/// Safe wrapper for coroutine states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoroutineState {
    Dead,
    Normal,
    Running,
    Suspended,
}

impl CoroutineState {
    fn from_raw(state: ffi::mco_state) -> Self {
        match state {
            ffi::mco_state_MCO_NORMAL => CoroutineState::Normal,
            ffi::mco_state_MCO_RUNNING => CoroutineState::Running,
            ffi::mco_state_MCO_SUSPENDED => CoroutineState::Suspended,
            _ => CoroutineState::Dead, // fallback for MCO_DEAD and unknown states
        }
    }
}

/// Safe wrapper for coroutine errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CoroutineError {
    #[error("No error")]
    Success,
    #[error("Generic error")]
    GenericError,
    #[error("Invalid pointer")]
    InvalidPointer,
    #[error("Invalid coroutine")]
    InvalidCoroutine,
    #[error("Coroutine not suspended")]
    NotSuspended,
    #[error("Coroutine not running")]
    NotRunning,
    #[error("Make context error")]
    MakeContextError,
    #[error("Switch context error")]
    SwitchContextError,
    #[error("Not enough space")]
    NotEnoughSpace,
    #[error("Out of memory")]
    OutOfMemory,
    #[error("Invalid arguments")]
    InvalidArguments,
    #[error("Invalid operation")]
    InvalidOperation,
    #[error("Stack overflow")]
    StackOverflow,
    #[error("Unknown error")]
    Unknown,
}

impl CoroutineError {
    fn from_raw(result: ffi::mco_result) -> Self {
        match result {
            ffi::mco_result_MCO_SUCCESS => CoroutineError::Success,
            ffi::mco_result_MCO_GENERIC_ERROR => CoroutineError::GenericError,
            ffi::mco_result_MCO_INVALID_POINTER => CoroutineError::InvalidPointer,
            ffi::mco_result_MCO_INVALID_COROUTINE => CoroutineError::InvalidCoroutine,
            ffi::mco_result_MCO_NOT_SUSPENDED => CoroutineError::NotSuspended,
            ffi::mco_result_MCO_NOT_RUNNING => CoroutineError::NotRunning,
            ffi::mco_result_MCO_MAKE_CONTEXT_ERROR => CoroutineError::MakeContextError,
            ffi::mco_result_MCO_SWITCH_CONTEXT_ERROR => CoroutineError::SwitchContextError,
            ffi::mco_result_MCO_NOT_ENOUGH_SPACE => CoroutineError::NotEnoughSpace,
            ffi::mco_result_MCO_OUT_OF_MEMORY => CoroutineError::OutOfMemory,
            ffi::mco_result_MCO_INVALID_ARGUMENTS => CoroutineError::InvalidArguments,
            ffi::mco_result_MCO_INVALID_OPERATION => CoroutineError::InvalidOperation,
            ffi::mco_result_MCO_STACK_OVERFLOW => CoroutineError::StackOverflow,
            _ => CoroutineError::Unknown,
        }
    }
}

/// Get the currently running coroutine (if any)
#[must_use]
pub fn running() -> Option<*mut ffi::mco_coro> {
    let co = unsafe { ffi::mco_running() };
    if co.is_null() { None } else { Some(co) }
}

/// Yield the current coroutine (safe version)
///
/// This function can be safely called from within a coroutine context.
/// It automatically detects if called from within a coroutine and yields appropriately.
///
/// # Errors
///
/// Returns `CoroutineError` if yielding fails or if not called from within a coroutine
pub fn yield_current() -> Result<(), CoroutineError> {
    if let Some(co) = running() {
        let result = unsafe { ffi::mco_yield(co) };
        if result == ffi::mco_result_MCO_SUCCESS {
            Ok(())
        } else {
            Err(CoroutineError::from_raw(result))
        }
    } else {
        Err(CoroutineError::InvalidCoroutine)
    }
}

/// Yield the current coroutine (unsafe version for advanced use)
///
/// # Safety
///
/// This must be called from within a coroutine context
///
/// # Errors
///
/// Returns `CoroutineError` if yielding fails or if not called from within a coroutine
pub unsafe fn yield_current_unsafe() -> Result<(), CoroutineError> {
    if let Some(co) = running() {
        let result = unsafe { ffi::mco_yield(co) };
        if result == ffi::mco_result_MCO_SUCCESS {
            Ok(())
        } else {
            Err(CoroutineError::from_raw(result))
        }
    } else {
        Err(CoroutineError::InvalidCoroutine)
    }
}
