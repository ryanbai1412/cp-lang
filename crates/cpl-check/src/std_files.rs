//! Finding and reading `std` modules: shared by build.rs, which bundles
//! them into the compiler, and by `CPL_STD_DIR`, which reads them from disk.

use std::path::{Path, PathBuf};

/// Every `.cp` file under `dir`, as `below` and its path below `dir`
/// without the extension (`"seq"`, `"geo/hull"`), and where it is.
pub fn collect(dir: &Path, below: &str, modules: &mut Vec<(String, PathBuf)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if path.is_dir() {
            collect(&path, &format!("{below}{file_name}/"), modules);
        } else if let Some(stem) = file_name.strip_suffix(".cp") {
            let name = format!("{below}{stem}");
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
/// whose body is on the lines below gets `todo` for it; the other lines
/// are blank, so a line number in the outline is the file's. A fn with
/// its return type written keeps `= todo` for a one-line body, and `todo`
/// for an `=` body on the next line. An `@lint` line stays, a method's
/// too; one whose `to` does not name its fn keeps the fn whole, since
/// such a `to` is checked against the body.
pub fn outline(text: &str) -> String {
    let mut out = String::with_capacity(text.len() / 4);
    let mut open = 0i32;
    let mut header: Option<usize> = None;
    let mut body: Option<usize> = None;
    let (mut tos, mut whole) = (Vec::new(), None::<usize>);
    for line in text.split_inclusive('\n') {
        let code = line.trim_end_matches(['\n', '\r']);
        let trimmed = code.trim_start();
        let comment = trimmed.starts_with("//");
        let top = !trimmed.is_empty() && trimmed.len() == code.len() && !comment;
        let declares = code.contains("fn ") && !comment;
        let indent = code.len() - trimmed.len();
        if whole.is_some_and(|at| trimmed.is_empty() || comment || indent > at) {
            out.push_str(if comment { &line[code.len()..] } else { line });
            continue;
        }
        whole = None;
        if trimmed.starts_with("@lint(") {
            out.push_str(line);
            tos.extend(
                trimmed
                    .split_once("to: \"")
                    .and_then(|(_, to)| to.split('"').next()),
            );
            continue;
        }
        if declares && !tos.is_empty() {
            let name = code.split(['(', '<']).next().unwrap_or_default();
            let name = name.rsplit(' ').next().unwrap_or_default();
            if tos.drain(..).any(|to| !to.contains(name)) {
                out.push_str(line);
                whole = Some(indent);
                continue;
            }
        }
        if top {
            open = 0;
        }
        if open == 0 && declares && nesting(code) == 0 {
            if let Some(head) = typed_head(code) {
                // `-> T =` with its body below: the body's line gets `todo`, so a
                // scan reads the head as the file's.
                if code[head.len()..].trim() == "=" {
                    out.push_str(line);
                    body = Some(code.len() - trimmed.len());
                    continue;
                }
                out.push_str(head);
                out.push_str(" = todo");
                out.push_str(&line[code.len()..]);
                continue;
            }
        }
        if comment {
            // Docs reach hover through cpl-doc, never through the outline.
            if line.ends_with('\n') {
                out.push('\n');
            }
            continue;
        }
        if open > 0 || top || declares {
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

/// `fn f(..) -> T` of a fn `line` writes as `fn f(..) -> T = body`: with
/// the type written, the body says nothing a scan reads.
fn typed_head(line: &str) -> Option<&str> {
    let mut depth = 0i32;
    let mut typed = false;
    for (at, c) in line.char_indices() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            _ if depth != 0 => {}
            '-' if line[at..].starts_with("->") => typed = true,
            '=' if typed
                && (line[at..].starts_with("= ") || line[at..].trim_end() == "=")
                && line[..at].ends_with(' ') =>
            {
                return Some(line[..at].trim_end());
            }
            _ => {}
        }
    }
    None
}

/// How many more brackets `line` opens than it closes.
pub(crate) fn nesting(line: &str) -> i32 {
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

/// The words of `texts` a `.name` can be, sorted, each once: the names
/// `call_table` gives counts for, in order, so it need not write them.
pub fn call_names<'a>(texts: impl IntoIterator<Item = &'a str>) -> Vec<&'a str> {
    let mut names = std::collections::BTreeSet::new();
    for text in texts {
        let bytes = text.as_bytes();
        let word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
        let mut at = 0;
        while at < bytes.len() {
            if !word(bytes[at]) {
                at += 1;
                continue;
            }
            let start = at;
            while at < bytes.len() && word(bytes[at]) {
                at += 1;
            }
            if bytes[start].is_ascii_lowercase() || bytes[start] == b'_' {
                names.insert(&text[start..at]);
            }
        }
    }
    names.into_iter().collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_typed_fn_whose_body_is_on_the_next_line_keeps_its_signature() {
        let text = "pub fn f(x: Int) -> Int =\n    x + 1\n";
        assert_eq!(
            super::outline(text),
            "pub fn f(x: Int) -> Int =\n    todo\n"
        );
    }

    /// An `@lint` whose `to` writes its fn's body, not its name, keeps the
    /// fn whole in the outline, at a struct's method too; other bodies,
    /// one named by its `@lint` among them, become `todo`, line for line.
    #[test]
    fn an_outline_keeps_a_fn_whole_whose_lint_writes_its_body() {
        let text = "@lint(l, from: \"a + 0\", to: \"a + 1\")\npub fn f(a: Int) -> Int:\n    // why\n    a + 1\npub fn g() -> Int:\n    2\npub struct S:\n    x: Int\n    @lint(m, from: \"s.x\", to: \"s.h\")\n    pub fn h(self) -> Int:\n        self.x\n    @lint(n, from: \"s.x == 0\", to: \"s.x != 0\")\n    pub fn k(self) -> Bool:\n        self.x != 0\n";
        let want = "@lint(l, from: \"a + 0\", to: \"a + 1\")\npub fn f(a: Int) -> Int:\n\n    a + 1\npub fn g() -> Int:\n    todo\npub struct S:\n\n    @lint(m, from: \"s.x\", to: \"s.h\")\n    pub fn h(self) -> Int:\n        todo\n    @lint(n, from: \"s.x == 0\", to: \"s.x != 0\")\n    pub fn k(self) -> Bool:\n        self.x != 0\n";
        assert_eq!(super::outline(text), want);
    }
}
