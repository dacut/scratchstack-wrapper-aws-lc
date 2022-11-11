#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, dead_code)]
#![allow(clippy::all)]

//! Rust wrapper for the `aws-lc` general cryptographic library. For testing purposes only.
//! For interacting with AWS services, use the `aws-sdk-rust` crate instead.

use libc::{time_t, timeval, FILE};
pub type va_list = *mut std::ffi::c_void;

#[cfg(unix)]
use libc::pthread_rwlock_t;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
