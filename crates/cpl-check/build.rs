//! Puts the text of every `std` module into the compiler, so `use
//! std.mod` resolves the same everywhere: the CLI, the editor and the
//! browser, with no folder to find at run time.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

#[path = "src/std_bundle.rs"]
mod std_bundle;
#[path = "src/std_files.rs"]
mod std_files;
use std_bundle::{browser_index, call_table, without_comments};
use std_files::{collect, outline, without_tests, EMBEDDED};

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("set by cargo"));
    println!("cargo:rerun-if-changed=build.rs");
    // `outline` and the rest of the modules `#[path]` puts in: their edits change what is written.
    println!("cargo:rerun-if-changed=src/std_files.rs");
    println!("cargo:rerun-if-changed=src/std_bundle.rs");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("set by cargo"));
    let std = manifest.join("../../std");
    let wasm = std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32");
    let dev = std::env::var("PROFILE").as_deref() == Ok("debug") && !wasm;
    let source = if dev {
        std.canonicalize().unwrap_or_else(|_| std.clone())
    } else {
        PathBuf::new()
    };
    println!("cargo:rustc-env=CPL_STD_SOURCE={}", source.display());
    // The outlines index is built here in every profile, so std/ is tracked
    // even when a dev build reads the module text at run time.
    bundle(&std, &out, "std", "std", true);
}

/// Writes `<prefix>_modules.rs`, the table of every module under `root`,
/// and `<prefix>_index.rs`, their outlines.
fn bundle(root: &Path, out: &Path, folder: &str, prefix: &str, track: bool) {
    if track {
        println!("cargo:rerun-if-changed={}", root.display());
    }
    let mut modules = Vec::new();
    collect(root, "", &mut modules);
    modules.sort();
    let wasm = std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32");
    // `lazy-std`: a browser build carries `EMBEDDED` whole and is given the
    // other modules (`GivenStd::give`).
    let lazy = std::env::var_os("CARGO_FEATURE_LAZY_STD").is_some();
    let mut table = String::from("&[\n");
    let mut index = String::from("&[\n");
    // Each module's whole text, which `std_calls` counts, and the text its
    // index entry carries, whose words `call_table` keys the counts on.
    let mut sources = Vec::new();
    let mut indexed = Vec::new();
    for (name, path) in &modules {
        if track {
            println!("cargo:rerun-if-changed={}", path.display());
        }
        let text = std::fs::read_to_string(path).expect("a readable module");
        let bundled = out.join(folder).join(format!("{name}.cp"));
        std::fs::create_dir_all(bundled.parent().expect("under OUT_DIR"))
            .expect("a writable OUT_DIR");
        sources.push(text.clone());
        let text = without_tests(&text);
        if wasm {
            if !lazy || EMBEDDED.contains(&name.as_str()) {
                let whole = if lazy {
                    browser_index(name, sources.last().expect("pushed above"))
                } else {
                    without_comments(&text)
                };
                std::fs::write(&bundled, &whole).expect("a writable OUT_DIR");
                indexed.push(whole);
                writeln!(
                    table,
                    "    ({name:?}, include_str!({:?})),",
                    bundled.display().to_string()
                )
                .expect("writing to a string");
                // It carries the text already, and the scans read the
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
        let outlined_text = if wasm {
            browser_index(name, sources.last().expect("pushed above"))
        } else {
            outline(&text)
        };
        std::fs::write(&outlined, &outlined_text).expect("a writable OUT_DIR");
        indexed.push(outlined_text);
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
    let calls = call_table(
        sources.iter().map(String::as_str),
        indexed.iter().map(String::as_str),
    );
    std::fs::write(
        out.join(format!("{prefix}_calls.rs")),
        format!("{calls:?}\n"),
    )
    .expect("a writable OUT_DIR");
}
