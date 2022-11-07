use {
    bindgen::builder,
    std::{env::var, fs::read_dir},
};

fn main() {
    let out_dir = var("OUT_DIR").expect("OUT_DIR not set");

    let aws_crt_prefix = match var("AWS_CRT_PREFIX") {
        Ok(aws_crt_prefix) => {
            println!("cargo:rustc-link-search={}/lib", aws_crt_prefix);
            aws_crt_prefix
        }
        Err(_) => "/usr/local".to_string(),
    };

    println!("cargo:rustc-link-lib=crypto");
    println!("cargo:rustc-link-lib=ssl");

    let mut builder = builder()
        .clang_arg(format!("-I{aws_crt_prefix}/include", aws_crt_prefix = aws_crt_prefix))
        .derive_debug(true)
        .derive_default(true)
        .derive_partialeq(true)
        .derive_eq(true);

    let dir = format!("{aws_crt_prefix}/include/openssl");
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
