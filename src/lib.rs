// In a file like `src/sys.rs` or inside a `mod sys { ... }` block.

#![allow(non_camel_case_types)] // This one is okay for FFI types.

use core::ffi::{c_char, c_void}; // Use standard C types

pub const MCO_DEFAULT_STORAGE_SIZE: usize = 1024;
pub const MCO_MIN_STACK_SIZE: usize = 32768;
pub const MCO_DEFAULT_STACK_SIZE: usize = 57344;

#[repr(C)]
#[derive(Debug)] // Add Debug for easy printing
pub struct mco_coro {
    _opaque: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum mco_result {
    MCO_SUCCESS = 0,
    MCO_GENERIC_ERROR,
    MCO_INVALID_POINTER,
    MCO_INVALID_COROUTINE,
    MCO_NOT_SUSPENDED,
    MCO_NOT_RUNNING,
    MCO_MAKE_CONTEXT_ERROR,
    MCO_SWITCH_CONTEXT_ERROR,
    MCO_NOT_ENOUGH_SPACE,
    MCO_OUT_OF_MEMORY,
    MCO_INVALID_ARGUMENTS,
    MCO_INVALID_OPERATION,
    MCO_STACK_OVERFLOW,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum mco_state {
    MCO_DEAD = 0,
    MCO_NORMAL,
    MCO_RUNNING,
    MCO_SUSPENDED,
}

#[repr(C)]
pub struct mco_desc {
    // Correctly use extern "C" for function pointers in FFI structs
    pub func: Option<unsafe extern "C" fn(co: *mut mco_coro)>,
    pub user_data: *mut c_void,
    pub malloc_cb: Option<unsafe extern "C" fn(size: usize, allocator_data: *mut c_void) -> *mut c_void>,
    pub free_cb: Option<unsafe extern "C" fn(ptr: *mut c_void, allocator_data: *mut c_void)>,
    pub allocator_data: *mut c_void,
    pub storage_size: usize,
    pub coro_size: usize,
    pub stack_size: usize,
}

// All pointers passed to C should be *mut if C might modify the pointed-to data,
// even if the function signature is `const`. This is a common C pattern that's
// tricky for Rust. We'll treat `mco_coro` as mutable internally.
#[link(name = "minicoro", kind = "static")]
unsafe extern "C" {
    // This function doesn't exist in minicoro.h, let's use the one that does: mco_init_desc
    // pub fn mco_desc_init(func: unsafe extern "C" fn(co: *mut mco_coro), stack_size: usize) -> mco_desc;
    pub fn mco_create(out_co: *mut *mut mco_coro, desc: *const mco_desc) -> mco_result;
    pub fn mco_destroy(co: *mut mco_coro) -> mco_result;
    pub fn mco_resume(co: *mut mco_coro) -> mco_result;
    pub fn mco_yield(co: *mut mco_coro) -> mco_result;
    pub fn mco_status(co: *mut mco_coro) -> mco_state;
    pub fn mco_get_user_data(co: *mut mco_coro) -> *mut c_void;
    pub fn mco_push(co: *mut mco_coro, src: *const c_void, len: usize) -> mco_result;
    pub fn mco_pop(co: *mut mco_coro, dest: *mut c_void, len: usize) -> mco_result;
    pub fn mco_peek(co: *mut mco_coro, dest: *mut c_void, len: usize) -> mco_result;
    pub fn mco_get_bytes_stored(co: *mut mco_coro) -> usize;
    pub fn mco_get_storage_size(co: *mut mco_coro) -> usize;
    pub fn mco_running() -> *mut mco_coro;
    pub fn mco_result_description(res: mco_result) -> *const c_char;
}

/// Initialize a coroutine descriptor with the given function and stack size.
///
/// # Safety
///
/// This function is unsafe because:
/// - The `func` parameter must be a valid function pointer that can safely handle
///   the coroutine lifecycle and properly manage the coroutine's execution context
/// - The caller must ensure that `func` does not perform undefined behavior when
///   called with a valid `mco_coro` pointer
/// - The `stack_size` should be large enough to accommodate the coroutine's
///   execution needs (at least `MCO_MIN_STACK_SIZE`)
pub unsafe fn mco_init_desc(
    func: unsafe extern "C" fn(co: *mut mco_coro),
    stack_size: usize
) -> mco_desc {
    mco_desc {
        func: Some(func),
        user_data: core::ptr::null_mut(),
        malloc_cb: None,
        free_cb: None,
        allocator_data: core::ptr::null_mut(),
        storage_size: 0,
        coro_size: 0,
        stack_size,
    }
}