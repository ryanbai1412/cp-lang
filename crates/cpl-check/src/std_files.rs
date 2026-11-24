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

/// The modules a browser build carries whole; it is given the others'
/// text when a program first needs them (`modules::provide_std`).
pub const EMBEDDED: &[&str] = &["mod", "prelude"];

/// What every scan of std's declarations reads, line for line with `text`:
/// each top-level line of code, each line that declares a `fn` (a `///`
/// doc aside), and the rest of a header still open on its next line. A fn
/// whose body is on the lines below gets `todo` for it, and a kept line
/// its `///` doc's first line, just above it, for hover; the other lines
/// are blank, so a line number in the outline is the file's.
pub fn outline(text: &str) -> String {
    let mut out = String::with_capacity(text.len() / 4);
    let mut open = 0i32;
    let mut header: Option<usize> = None;
    let mut body: Option<usize> = None;
    // A `///` block's first line and how many lines the block has.
    let mut doc: Option<(&str, usize)> = None;
    for line in text.split_inclusive('\n') {
        let code = line.trim_end_matches(['\n', '\r']);
        let trimmed = code.trim_start();
        if open == 0 && trimmed.starts_with("///") {
            doc.get_or_insert((code, 0)).1 += 1;
            continue;
        }
        let above = doc.take();
        let comment = trimmed.starts_with("//");
        let top = !trimmed.is_empty() && trimmed.len() == code.len() && !comment;
        let declares = code.contains("fn ") && !trimmed.starts_with("///");
        if top {
            open = 0;
        }
        if open > 0 || top || declares {
            if let Some((first, lines)) = above {
                out.push_str(&"\n".repeat(lines - 1));
                out.push_str(first);
                out.push('\n');
            }
            out.push_str(line);
            if !comment {
                if declares && open == 0 {
                    header = Some(code.len() - trimmed.len());
                }
                open = (open + nesting(code)).max(0);
                body = header.filter(|_| open == 0 && code.trim_end().ends_with(':'));
                if open == 0 {
                    header = None;
                }
            }
            continue;
        }
        if let Some((_, lines)) = above {
            out.push_str(&"\n".repeat(lines));
        }
        if !trimmed.is_empty() && !comment {
            if let Some(indent) = body.take() {
                out.push_str(&" ".repeat(indent + 4));
                out.push_str("todo");
            }
        }
        if line.ends_with('\n') {
            out.push('\n');
        }
    }
    out
}

/// How many more brackets `line` opens than it closes.
fn nesting(line: &str) -> i32 {
    line.chars()
        .map(|c| match c {
            '(' | '[' => 1,
            ')' | ']' => -1,
            _ => 0,
        })
        .sum()
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
