//! C ABI for UIFY. All exports use `extern "C"`, no panics cross the
//! boundary, and every resource is explicitly owned (create / destroy pairs).
//!
//! Header is generated at build time by `cbindgen` into `include/uify.h`
//! so non-Rust consumers can integrate without touching Cargo.
//!
//! TODO: implement.

#![forbid(unsafe_op_in_unsafe_fn)]

/// Placeholder.
#[no_mangle]
pub extern "C" fn uify_abi_version() -> u32 {
    0
}
