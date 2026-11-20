//! Puts the text of every `stdlib/` module into the compiler, so `use
//! std.mod` resolves where there is no `stdlib/` folder to read: in the
//! browser, and for a `cpl` binary that was copied somewhere on its own.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("set by cargo"));
    let stdlib = manifest.join("../../stdlib");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", stdlib.display());
    let mut modules = Vec::new();
    collect(&stdlib, &stdlib, &mut modules);
    modules.sort();
    let mut table = String::from("&[\n");
    for (name, path) in &modules {
        println!("cargo:rerun-if-changed={}", path.display());
        writeln!(table, "    ({name:?}, include_str!({:?})),", path.display().to_string())
            .expect("writing to a string");
    }
    table.push_str("]\n");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("set by cargo"));
    std::fs::write(out.join("std_modules.rs"), table).expect("a writable OUT_DIR");
}

/// Every `.cp` file under `dir`, as its path below `root` without the
/// extension (`"seq"`, `"geo/hull"`) and where it is.
fn collect(root: &Path, dir: &Path, modules: &mut Vec<(String, PathBuf)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(root, &path, modules);
        } else if path.extension().is_some_and(|ext| ext == "cp") {
            let below = path.strip_prefix(root).expect("found under root").with_extension("");
            let name = below
                .components()
                .map(|part| part.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            let path = path.canonicalize().unwrap_or(path);
            modules.push((name, path));
        }
    }
}
