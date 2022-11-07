use {
    bindgen::builder,
    std::{env::var, fs::read_dir},
};

fn main() {
    let root = var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let out_dir = var("OUT_DIR").expect("OUT_DIR not set");

    if let Ok(aws_crt_prefix) = var("AWS_CRT_PREFIX") {
        println!("cargo:rustc-link-search={aws_crt_prefix}/lib");
    }

    println!("cargo:rustc-link-lib=crypto");
    println!("cargo:rustc-link-lib=ssl");

    let mut builder = builder()
        .clang_arg(format!("-I{root}/include"))
        .derive_debug(true)
        .derive_default(true)
        .derive_partialeq(true)
        .derive_eq(true);

    let dir = format!("{root}/include/openssl");
    let mut n_includes = 0;

    for entry in read_dir(&dir).expect("Unable to list header files in include/openssl") {
        let entry = entry.expect("Unable to read directory entry in include/openssl");

        if entry.file_type().expect("Unable to read file type").is_file() {
            let file_name_string = entry.file_name();
            if let Some(file_name_utf8) = file_name_string.to_str() {
                if file_name_utf8.ends_with(".h") {
                    builder = builder.header(format!("{dir}/{file_name_utf8}"));
                    n_includes += 1;
                }
            }
        }
    }

    if n_includes == 0 {
        panic!("No header files found in include/aws/common");
    }

    builder = builder.allowlist_function(".*").allowlist_type(".*").allowlist_var(".*");

    let bindings = builder.generate().expect("Unable to generate bindings");
    bindings.write_to_file(format!("{out_dir}/bindings.rs")).expect("Failed to write bindings.");

    if cfg!(any(target_os = "ios", target_os = "macos")) {
        println!("cargo:rustc-link-arg=-framework");
        println!("cargo:rustc-link-arg=CoreFoundation");
    }
}
