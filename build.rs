use {
    bindgen::builder,
    std::{
        env::var,
        fs::{copy, create_dir_all, read_dir},
        path::{Path, PathBuf},
    },
};

const LINK_LIBS: &str = r#"
crypto
ssl
"#;
const INCLUDE_PATH: &str = "openssl";
const DEP_LIBRARIES: &str = "";
const FUNCTIONS: &str = r#"
AES.*
ASN1.*
BIO.*
BORINGSSL.*
BN.*
CMAC.*
CRYPTO.*
CTR.*
DES.*
d2i.*
DH.*
DSA.*
EC.*
ED25519.*
ERR.*
EVP.*
FIPS.*
HMAC.*
HKDF.*
HRSS.*
i2d.*
i2o.*
o2i.*
NCONF.*
MD4.*
MD5.*
OBJ.*
OCSP.*
PEM.*
PKCS.*
RAND.*
RC4.*
RIPEMD.*
RSA.*
SPAKE2.*
SHA.*
SIPHASH.*
SSHKDF
SSL.*
TLS.*
TRUST.*
DTLS.*
"#;

const TYPES: &str = r#"
ACCESS_DESCRIPTION_st
ACCESS_DESCRIPTION
aes.*
AES.*
asn1.*
ASN1.*
AUTHORITY.*
BASIC_CONSTRAINTS.*
bignum_ctx
bignum_st
BIGNUM
bio.*
BIO.*
bn_.*
BN_.*
BUF_MEM
buf_mem_st
cbb.*
CBB.*
cbs_st
CBS.*
cmac.*
CERTIFICATEPOLICIES
COMP_METHOD
conf_st
CONF
CRL_DIST.*
CRYPTO.*
crypto.*
CMAC.*
ctr_.*
CTR.*
d2i_of_void
DES.*
dh_st
DH
DIST_POINT.*
dsa.*
DSA.*
ec.*
EC.*
EDIPartyName_st
EDIPARTYNAME
ENGINE.*
engine.*
ERR.*
env_md_ctx_st
env_md_st
evp.*
EVP.*
EXTENDED_KEY_USAGE
fips.*
GENERAL_NAME.*
hmac.*
Hmac.*
HMAC.*
HRSS.*
i2d_of_void
ISSUING_DIST_POINT.*
openssl_.*
md4.*
md5.*
md_.*
MD4.*
MD5.*
Netscape.*
NETSCAPE.*
NOTICEREF.*
ocsp.*
OCSP.*
otherName_st
OTHERNAME
pem.*
point_conversion_form_t
POLICYINFO.*
POLICYQUALINFO.*
poly1305.*
PROXY.*
pkcs.*
PKCS.*
private_key_st
rand.*
RAND.*
rc4.*
RC4.*
RIPEMD.*
rsa.*
RSA.*
sha.*
SHA.*
spake.*
SPAKE.*
srtp.*
SRTP.*
SSL.*
ssl.*
st_ERR.*
stack_st_.*
trust.*
TRUST.*
USERNOTICE.*
v3_ext_ctx
v3_ext_method
X25519.*
X509.*
x509.*
"#;

const VARS: &str = "";

fn get_include_dir<P: AsRef<Path>>(dir: P) -> PathBuf {
    let mut result = PathBuf::from(dir.as_ref());

    for folder in INCLUDE_PATH.split('/') {
        result.push(folder);
    }

    result
}

fn main() {
    let root = PathBuf::from(var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let out_dir = PathBuf::from(var("OUT_DIR").expect("OUT_DIR not set"));

    let src_include_dir = root.join("include");
    let dst_include_dir = out_dir.join("include");
    let src_lib_include_dir = get_include_dir(&src_include_dir);
    let dst_lib_include_dir = get_include_dir(&dst_include_dir);
    let src_include_dir_str = src_include_dir.to_string_lossy();
    let dst_include_dir_str = dst_include_dir.to_string_lossy();
    let src_lib_include_dir_str = src_lib_include_dir.to_string_lossy();
    let dst_lib_include_dir_str = dst_lib_include_dir.to_string_lossy();

    println!("cargo:include={dst_include_dir_str}");
    println!("cargo:rerun-if-changed=include");
    println!("cargo:rerun-if-env-changed=AWS_CRT_PREFIX");

    if let Ok(aws_crt_prefix) = var("AWS_CRT_PREFIX") {
        println!("cargo:rustc-link-search={aws_crt_prefix}/lib");
    }

    for library_name in LINK_LIBS.split('\n') {
        let library_name = library_name.trim();
        if !library_name.is_empty() {
            println!("cargo:rustc-link-lib={library_name}");
        }
    }

    // Copy include files over
    create_dir_all(&dst_lib_include_dir)
        .unwrap_or_else(|e| panic!("Unable to create directory {dst_lib_include_dir_str}: {e}"));

    let mut builder = builder()
        .clang_arg(format!("-I{src_include_dir_str}"))
        .derive_debug(true)
        .derive_default(true)
        .derive_partialeq(true)
        .derive_eq(true)
        .allowlist_recursively(false); // Don't dive into dependent libraries.
    
    for dep in DEP_LIBRARIES.split('\n') {
        let dep = dep.trim();
        if dep.is_empty() {
            continue;
        }

        let dep = String::from(dep).replace('-', "_").to_uppercase();
        let dep_include_dir = PathBuf::from(var(format!("DEP_{dep}_INCLUDE")).unwrap_or_else(|_| panic!("DEP_{dep}_INCLUDE not set")));
        let dep_include_dir_str = dep_include_dir.to_string_lossy();
        builder = builder.clang_arg(format!("-I{dep_include_dir_str}"));
    }

    let mut n_includes = 0;

    for entry in read_dir(&src_lib_include_dir)
        .unwrap_or_else(|e| panic!("Unable to list header files in {src_lib_include_dir_str}: {e}"))
    {
        let entry =
            entry.unwrap_or_else(|e| panic!("Unable to read directory entry in {src_lib_include_dir_str}: {e}"));
        let file_name_string = entry.file_name();
        let src_path = src_lib_include_dir.join(&file_name_string);
        let src_path_str = src_path.to_string_lossy();
        let dst_path = dst_lib_include_dir.join(&file_name_string);

        if entry.file_type().unwrap_or_else(|e| panic!("Unable to read file type of {src_path_str}: {e}")).is_file() {
            // Only include header files ending with .h; ignore .inl.
            let file_name_utf8 = file_name_string.to_str().expect("Unable to convert file name to UTF-8");
            if file_name_utf8.ends_with(".h") {
                builder = builder.header(src_path_str.to_string());
                n_includes += 1;
            }

            // Copy all files to the output directory.
            copy(&src_path, &dst_path).unwrap_or_else(|e| {
                panic!(
                    "Failed to copy from {src_path_str} to {dst_path_str}: {e}",
                    dst_path_str = dst_path.to_string_lossy()
                )
            });
        }
    }

    if n_includes == 0 {
        panic!("No header files found in {src_lib_include_dir_str}");
    }

    for function_pattern in FUNCTIONS.split('\n') {
        let function_pattern = function_pattern.trim();
        if !function_pattern.is_empty() {
            builder = builder.allowlist_function(function_pattern);
        }
    }

    for type_pattern in TYPES.split('\n') {
        let type_pattern = type_pattern.trim();
        if !type_pattern.is_empty() {
            builder = builder.allowlist_type(type_pattern);
        }
    }

    for var_pattern in VARS.split('\n') {
        let var_pattern = var_pattern.trim();
        if !var_pattern.is_empty() {
            builder = builder.allowlist_var(var_pattern);
        }
    }

    let bindings_filename = out_dir.join("bindings.rs");
    let bindings = builder.generate().expect("Unable to generate bindings");
    bindings.write_to_file(&bindings_filename).unwrap_or_else(|e| {
        panic!(
            "Failed to write bindings to {bindings_filename_str}: {e}",
            bindings_filename_str = bindings_filename.to_string_lossy()
        )
    });

    if cfg!(any(target_os = "ios", target_os = "macos")) {
        println!("cargo:rustc-link-arg=-framework");
        println!("cargo:rustc-link-arg=CoreFoundation");
    }
}
