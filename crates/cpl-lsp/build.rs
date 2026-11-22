//! The names of the `spec/std/` modules, for `use std.` completion: the
//! loader can read a module it is asked for but cannot list them.

use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("set by cargo"));
    let stdlib = manifest.join("../../spec/std");
    println!("cargo:rerun-if-changed={}", stdlib.display());
    let mut names: Vec<String> = std::fs::read_dir(&stdlib)
        .map(|dir| {
            dir.filter_map(|entry| {
                let path = entry.ok()?.path();
                let stem = path.file_stem()?.to_str()?.to_string();
                (path.extension()? == "cp" && stem != "prelude").then_some(stem)
            })
            .collect()
        })
        .unwrap_or_default();
    names.sort();
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("set by cargo"));
    std::fs::write(out.join("std_modules.rs"), format!("&{names:?}\n"))
        .expect("a writable OUT_DIR");
}
