//! Bakes `std/` into the binary as it was at build time, so a binary always checks against the
//! std it was built with, never the working tree's later edits (`CPL_NEXT_STD_DIR` names another).

use std::path::{Path, PathBuf};

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../std");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut files = Vec::new();
    walk(&root, "std", &mut files);
    files.sort();
    let mut out = String::from("&[\n");
    for (module, path) in files {
        println!("cargo:rerun-if-changed={}", path.display());
        out.push_str(&format!(
            "    ({module:?}, include_str!({:?})),\n",
            path.display().to_string()
        ));
    }
    out.push_str("]\n");
    let dest = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("std_files.rs");
    std::fs::write(dest, out).unwrap();
}

/// Each `.cp` file under `dir`, named by module (`std.graph`, `std.seq.sort`).
fn walk(dir: &Path, module: &str, files: &mut Vec<(String, PathBuf)>) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        let stem = path
            .file_stem()
            .map_or(String::new(), |s| s.to_string_lossy().into_owned());
        if path.is_dir() {
            walk(&path, &format!("{module}.{stem}"), files);
        } else if path.extension().is_some_and(|e| e == "cp") {
            files.push((
                format!("{module}.{stem}"),
                path.canonicalize().unwrap_or(path),
            ));
        }
    }
}
