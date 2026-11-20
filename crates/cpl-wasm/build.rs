//! Gives the WebAssembly module a stack sized for the interpreter.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_FAMILY").is_ok_and(|family| family.contains("wasm")) {
        // The linker's default is 1 MB. The interpreter recurses once per
        // call in the program, so it gets more; `STACK_BYTES` in
        // `src/lib.rs` stops a run before this is used up.
        println!("cargo:rustc-link-arg-cdylib=-zstack-size={}", 4 << 20);
    }
}
