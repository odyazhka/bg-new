fn main() {
    nasm_rs::compile_library("libsyscalls.a", &["src/syscalls.asm"]).unwrap();
    println!("cargo:rustc-link-lib=static=syscalls");
    println!("cargo:rustc-link-lib=dylib=X11");
}
