//! Finding and reading `std` modules: shared by build.rs, which bundles
//! them into the compiler, and by `CPL_STD_DIR`, which reads them from disk.

use std::path::{Path, PathBuf};

/// Every `.cp` file under `dir`, as its path below `root` without the
/// extension (`"seq"`, `"geo/hull"`) and where it is.
pub fn collect(root: &Path, dir: &Path, modules: &mut Vec<(String, PathBuf)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(root, &path, modules);
        } else if path.extension().is_some_and(|ext| ext == "cp") {
            let below = path
                .strip_prefix(root)
                .expect("found under root")
                .with_extension("");
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

pub fn without_tests(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_test = false;
    for line in text.split_inclusive('\n') {
        let at_top = !line.starts_with([' ', '\t', '\n', '\r']);
        if line.trim_end() == "@test" {
            in_test = true;
        } else if in_test && at_top && !line.starts_with("fn ") {
            in_test = false;
        }
        if in_test {
            out.push_str(if line.ends_with('\n') { "\n" } else { "" });
        } else {
            out.push_str(line);
        }
    }
    out
}
