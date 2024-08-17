fn main() {
    println!("cargo:rustc-link-arg=-owasm_export.js");
    println!("cargo:rustc-link-arg=-sNO_EXIT_RUNTIME=1");
    println!("cargo:rustc-link-arg=-sEXPORTED_RUNTIME_METHODS=['ccall', 'cwrap']");
    println!("cargo:rustc-link-arg=-sEXPORTED_FUNCTIONS=['_bin_to_img', '_img_to_bin', '_buf_alloc', '_buf_resize', '_buf_inner', '_buf_len', '_buf_free']");
    println!("cargo:rustc-link-arg=-sMODULARIZE=1");
    println!("cargo:rustc-link-arg=-sEXPORT_ES6=1");
}
