//! Puts the text of every `std` module into the compiler, so `use
//! std.mod` resolves the same everywhere: the CLI, the editor and the
//! browser, with no folder to find at run time.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

#[path = "src/std_files.rs"]
mod std_files;
use std_files::{collect, outline, public_names, without_tests, EMBEDDED};

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("set by cargo"));
    println!("cargo:rerun-if-changed=build.rs");
    // `outline` and the rest of the modules `#[path]` puts in: their edits change what is written.
    println!("cargo:rerun-if-changed=src/std_files.rs");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("set by cargo"));
    let std = manifest.join("../../std");
    // A dev build reads std/ from here at run time (`modules.rs`), so a std
    // edit rebuilds nothing; release and the browser use the copy put in.
    let wasm = std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32");
    let dev = std::env::var("PROFILE").as_deref() == Ok("debug") && !wasm;
    let source = if dev {
        std.canonicalize().unwrap_or_else(|_| std.clone())
    } else {
        PathBuf::new()
    };
    println!("cargo:rustc-env=CPL_STD_SOURCE={}", source.display());
    bundle(&std, &out, "std", "std", !dev);
}

/// Writes `<prefix>_modules.rs`, the table of every module under `root`,
/// and `<prefix>_names.rs`, every public name with its module.
fn bundle(root: &Path, out: &Path, folder: &str, prefix: &str, track: bool) {
    if track {
        println!("cargo:rerun-if-changed={}", root.display());
    }
    let mut modules = Vec::new();
    collect(root, root, &mut modules);
    modules.sort();
    let wasm = std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32");
    // `lazy-std`: a browser build carries `EMBEDDED` whole and is given the
    // other modules (`modules::provide_std`).
    let lazy = std::env::var_os("CARGO_FEATURE_LAZY_STD").is_some();
    let mut table = String::from("&[\n");
    let mut index = String::from("&[\n");
    for (name, path) in &modules {
        if track {
            println!("cargo:rerun-if-changed={}", path.display());
        }
        let text = std::fs::read_to_string(path).expect("a readable module");
        let bundled = out.join(folder).join(format!("{name}.cp"));
        std::fs::create_dir_all(bundled.parent().expect("under OUT_DIR"))
            .expect("a writable OUT_DIR");
        let text = without_tests(&text);
        if wasm {
            if !lazy || EMBEDDED.contains(&name.as_str()) {
                std::fs::write(&bundled, without_comments(&text)).expect("a writable OUT_DIR");
                writeln!(
                    table,
                    "    ({name:?}, include_str!({:?})),",
                    bundled.display().to_string()
                )
                .expect("writing to a string");
            }
            if !lazy {
                // It carries every text already, and the scans read the
                // same off it as off the outline.
                writeln!(
                    index,
                    "    ({name:?}, include_str!({:?})),",
                    bundled.display().to_string()
                )
                .expect("writing to a string");
                continue;
            }
        } else {
            std::fs::write(&bundled, &text).expect("a writable OUT_DIR");
            writeln!(
                table,
                "    ({name:?}, include_str!({:?})),",
                bundled.display().to_string()
            )
            .expect("writing to a string");
        }
        let outlined = out
            .join(format!("{folder}_index"))
            .join(format!("{name}.cp"));
        std::fs::create_dir_all(outlined.parent().expect("under OUT_DIR"))
            .expect("a writable OUT_DIR");
        std::fs::write(&outlined, outline(&text)).expect("a writable OUT_DIR");
        writeln!(
            index,
            "    ({name:?}, include_str!({:?})),",
            outlined.display().to_string()
        )
        .expect("writing to a string");
    }
    table.push_str("]\n");
    index.push_str("]\n");
    std::fs::write(out.join(format!("{prefix}_modules.rs")), table).expect("a writable OUT_DIR");
    std::fs::write(out.join(format!("{prefix}_index.rs")), index).expect("a writable OUT_DIR");
    std::fs::write(out.join(format!("{prefix}_names.rs")), std_names(&modules))
        .expect("a writable OUT_DIR");
    std::fs::write(out.join(format!("{prefix}_calls.rs")), std_calls(&modules))
        .expect("a writable OUT_DIR");
}

/// How often std's own text, its doc examples included, writes `.name`:
/// the `std_calls` table, by name, of the names written at least twice.
fn std_calls(modules: &[(String, PathBuf)]) -> String {
    let mut counts: std::collections::BTreeMap<String, u32> = Default::default();
    for (_, path) in modules {
        let text = std::fs::read_to_string(path).expect("a readable module");
        let bytes = text.as_bytes();
        for (at, _) in text.match_indices('.') {
            if at > 0 && bytes[at - 1] == b'.' {
                continue;
            }
            let name: String = text[at + 1..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if name.starts_with(|c: char| c.is_ascii_lowercase() || c == '_') {
                *counts.entry(name).or_default() += 1;
            }
        }
    }
    let mut table = String::from("&[\n");
    for (name, count) in counts.iter().filter(|(_, n)| **n >= 2) {
        writeln!(table, "    ({name:?}, {count}),").expect("writing to a string");
    }
    table.push_str("]\n");
    table
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

/// The `public_names` table as Rust source.
fn std_names(modules: &[(String, PathBuf)]) -> String {
    let texts: Vec<(&str, String)> = modules
        .iter()
        .map(|(module, path)| {
            let text = std::fs::read_to_string(path).expect("a readable module");
            (module.as_str(), text)
        })
        .collect();
    let mut table = String::from("&[\n");
    for (name, module) in public_names(texts.iter().map(|(m, t)| (*m, t.as_str()))) {
        writeln!(table, "    ({name:?}, {module:?}),").expect("writing to a string");
    }
    table.push_str("]\n");
    table
}
