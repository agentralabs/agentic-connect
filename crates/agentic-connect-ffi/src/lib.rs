//! FFI bindings for AgenticConnect — C-compatible interface.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Return the library version as a C string.
///
/// # Safety
/// The returned pointer is valid until the next call to this function.
#[no_mangle]
pub extern "C" fn acnx_version() -> *const c_char {
    let version = CString::new(env!("CARGO_PKG_VERSION")).unwrap_or_default();
    version.into_raw()
}

/// Free a string returned by this library.
///
/// # Safety
/// The pointer must have been returned by an `acnx_*` function.
#[no_mangle]
pub unsafe extern "C" fn acnx_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(unsafe { CString::from_raw(ptr) });
    }
}
