//! Puts the text of every `std` module into the compiler, so `use
//! std.mod` resolves the same everywhere: the CLI, the editor and the
//! browser, with no folder to find at run time.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("set by cargo"));
    println!("cargo:rerun-if-changed=build.rs");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("set by cargo"));
    bundle(&manifest.join("../../std"), &out, "std", "std");
}

/// Writes `<prefix>_modules.rs`, the table of every module under `root`,
/// and `<prefix>_names.rs`, every public name with its module.
fn bundle(root: &Path, out: &Path, folder: &str, prefix: &str) {
    println!("cargo:rerun-if-changed={}", root.display());
    let mut modules = Vec::new();
    collect(root, root, &mut modules);
    modules.sort();
    let mut table = String::from("&[\n");
    for (name, path) in &modules {
        println!("cargo:rerun-if-changed={}", path.display());
        let text = std::fs::read_to_string(path).expect("a readable module");
        let bundled = out.join(folder).join(format!("{name}.cp"));
        std::fs::create_dir_all(bundled.parent().expect("under OUT_DIR"))
            .expect("a writable OUT_DIR");
        let mut text = without_tests(&text);
        if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32") {
            text = without_comments(&text);
        }
        std::fs::write(&bundled, text).expect("a writable OUT_DIR");
        writeln!(
            table,
            "    ({name:?}, include_str!({:?})),",
            bundled.display().to_string()
        )
        .expect("writing to a string");
    }
    table.push_str("]\n");
    std::fs::write(out.join(format!("{prefix}_modules.rs")), table).expect("a writable OUT_DIR");
    std::fs::write(out.join(format!("{prefix}_names.rs")), std_names(&modules))
        .expect("a writable OUT_DIR");
}

/// `text` with each `@test` fn blanked to empty lines. A program that
/// imports a module never runs its tests, and `cpl test` reads the file
/// itself; keeping every line keeps spans into the module exact.
/// The browser build blanks whole-line comments: nothing it runs reads std's
/// docs, and they are half of std's compressed size. Blank lines keep every
/// line number, so std diagnostics point at the same line on every target.
fn without_comments(text: &str) -> String {
    text.split_inclusive('\n')
        .map(|line| {
            if line.trim_start().starts_with("//") {
                if line.ends_with('\n') {
                    "\n"
                } else {
                    ""
                }
            } else {
                line
            }
        })
        .collect()
}

fn without_tests(text: &str) -> String {
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

/// Every public name that exactly one `std` module declares, with
/// that module, sorted by name: what the hint for a missing `use` reads.
/// It is read off the files, so a new `pub fn` needs no second edit.
fn std_names(modules: &[(String, PathBuf)]) -> String {
    let mut found: std::collections::BTreeMap<String, Vec<&str>> = Default::default();
    for (module, path) in modules {
        let text = std::fs::read_to_string(path).expect("a readable module");
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
            if name.starts_with('_') {
                continue;
            }
            found.entry(name).or_default().push(module);
        }
    }
    let mut table = String::from("&[\n");
    for (name, modules) in &found {
        if let [module] = modules.as_slice() {
            writeln!(table, "    ({name:?}, {module:?}),").expect("writing to a string");
        }
    }
    table.push_str("]\n");
    table
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
