# scratchstack-wrapper-aws-lc
Rust wrappers for the [`aws-lc`](https://github.com/awslabs/aws-lc) general purpose cryptographic library.

Currently, this only provided the minimum set of bindings necessary for Scratchstack to verify interoperability
with AWS CRT authentication libraries.  This is not intended to be a complete set of bindings for the CRT. You
probably *do not want to use these bindings* in your own projects. If you need to communicate with AWS services,
use the [official AWS SDK for Rust](https://github.com/awslabs/aws-sdk-rust) instead.

Documentation: https://docs.rs/scratchstack-wrapper-aws-lc
