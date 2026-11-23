//! Puts the text of every `std` module into the compiler, so `use
//! std.mod` resolves the same everywhere: the CLI, the editor and the
//! browser, with no folder to find at run time.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

#[path = "src/std_files.rs"]
mod std_files;
use std_files::{collect, public_names, without_tests};

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("set by cargo"));
    println!("cargo:rerun-if-changed=build.rs");
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
    let mut table = String::from("&[\n");
    for (name, path) in &modules {
        if track {
            println!("cargo:rerun-if-changed={}", path.display());
        }
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
