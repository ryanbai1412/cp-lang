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

/// Every public name that exactly one module declares, with that module,
/// sorted by name: what the hint for a missing `use` reads. It is read off
/// the files, so a new `pub fn` needs no second edit.
pub fn public_names<'a>(
    modules: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Vec<(String, String)> {
    let mut found: std::collections::BTreeMap<String, Vec<&str>> = Default::default();
    for (module, text) in modules {
        for line in text.lines() {
            let mut words = line.split_whitespace();
            let public = !line.starts_with(' ') && words.next() == Some("pub");
            let declares = matches!(
                words.next(),
                Some("fn" | "struct" | "enum" | "trait" | "type" | "const" | "domain")
            );
            if !public || !declares {
                continue;
            }
            let name: String = words
                .next()
                .unwrap_or_default()
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.starts_with('_') {
                found.entry(name).or_default().push(module);
            }
        }
    }
    found
        .into_iter()
        .filter_map(|(name, modules)| match modules.as_slice() {
            [module] => Some((name, module.to_string())),
            _ => None,
        })
        .collect()
}
