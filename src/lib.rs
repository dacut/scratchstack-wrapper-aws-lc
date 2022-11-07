#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, dead_code)]
#![warn(clippy::all)]

//! Rust wrapper for the `aws-c-common` library. For testing purposes only.
//! For interacting with AWS services, use the `aws-sdk-rust` crate instead.

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
