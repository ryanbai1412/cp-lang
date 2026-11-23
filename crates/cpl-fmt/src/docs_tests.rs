//! The ```` ```cp ```` blocks a reader sees in the handbook, spec, docs, std
//! notes and site are formatted the way `cpl fmt` formats their own code, so
//! copying an example never brings in a style the formatter would change.

use std::path::{Path, PathBuf};

fn docs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for path in entries.flatten().map(|e| e.path()) {
        if path.is_dir() && !path.ends_with("node_modules") {
            docs(&path, out);
        } else if path.extension().is_some_and(|e| e == "md" || e == "mdx") {
            out.push(path);
        }
    }
}

/// Each column-0 ```` ```cp ```` block of `text`, as its first line number and body.
fn blocks(text: &str) -> Vec<(usize, String)> {
    let (mut out, mut open) = (Vec::new(), None::<(usize, String)>);
    for (i, line) in text.lines().enumerate() {
        match &mut open {
            Some(_) if line == "```" => out.push(open.take().unwrap()),
            Some((_, body)) => {
                body.push_str(line);
                body.push('\n');
            }
            None if line == "```cp"
                || line.starts_with("```cp ")
                || line.starts_with("```cp\t") =>
            {
                open = Some((i + 2, String::new()))
            }
            None => {}
        }
    }
    out
}

#[test]
fn every_doc_snippet_is_formatted() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = vec![root.join("README.md")];
    for dir in ["handbook", "spec", "docs", "std", "web/content"] {
        docs(&root.join(dir), &mut files);
    }
    let (mut seen, mut unformatted) = (0, Vec::new());
    for path in files {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        for (line, body) in blocks(&text) {
            seen += 1;
            if crate::format(&body).is_ok_and(|f| f != body) {
                let rel = path.strip_prefix(&root).unwrap_or(&path);
                unformatted.push(format!("{}:{line}", rel.display()));
            }
        }
    }
    assert!(seen > 400, "found only {seen} snippets");
    assert!(
        unformatted.is_empty(),
        "{} doc snippets differ from `cpl fmt` (format each block's code with `cpl fmt`):\n{}",
        unformatted.len(),
        unformatted.join("\n")
    );
}
