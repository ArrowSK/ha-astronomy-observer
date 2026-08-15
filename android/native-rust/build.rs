use std::env;

fn main() {
    let lib_dir = env::var("AO_ANDROID_C_LIB_DIR")
        .expect("AO_ANDROID_C_LIB_DIR must point to the target-specific Android C library");
    println!("cargo:rustc-link-search=native={lib_dir}");
    println!("cargo:rustc-link-lib=static=ao_android_c");
    println!("cargo:rustc-link-lib=m");
    println!("cargo:rerun-if-env-changed=AO_ANDROID_C_LIB_DIR");
}
