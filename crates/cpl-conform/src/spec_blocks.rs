use std::collections::HashMap;
use std::path::Path;

use cpl_check::Database;

use crate::runner;
use crate::scoreboard::{CaseOutcome, SuiteReport};

/// A short stable id for a block or example body: the first 10 hex digits of its FNV-1a hash.
pub(crate) fn block_hash(text: &str) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in text.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{h:012x}")[..10].to_string()
}

#[derive(Clone, Debug)]
pub struct SpecBlock {
    pub id: String,
    pub code: String,
    pub marker: Marker,
}

/// The `<!-- spec-only: reason -->`, `<!-- given: code -->` or
/// `<!-- rejects: E0608 -->` comment ending on the line right above a fence.
/// `given` code (top-level lines, one per line) is prepended when the block
/// runs; a spec-only block that fails to check is a skip, not an error; a
/// `rejects` block passes only when checking it reports that code.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Marker {
    #[default]
    None,
    SpecOnly(String),
    Given(String),
    Rejects(String),
}

fn marker_above(before: &str) -> Marker {
    let Some(body) = before
        .trim_end_matches([' ', '\t'])
        .strip_suffix('\n')
        .and_then(|b| b.trim_end_matches([' ', '\t']).strip_suffix("-->"))
    else {
        return Marker::None;
    };
    let Some(open) = body.rfind("<!--") else {
        return Marker::None;
    };
    let inner = body[open + 4..].trim_start();
    if let Some(r) = inner.strip_prefix("spec-only:") {
        Marker::SpecOnly(r.split_whitespace().collect::<Vec<_>>().join(" "))
    } else if let Some(c) = inner.strip_prefix("rejects:") {
        Marker::Rejects(c.trim().to_string())
    } else if let Some(c) = inner.strip_prefix("given:") {
        let lines: Vec<&str> = c.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
        Marker::Given(lines.join("\n"))
    } else {
        Marker::None
    }
}

/// Extracts fenced ```cp / ```python code blocks from markdown text.
pub fn extract_blocks(text: &str) -> Vec<String> {
    extract_marked_blocks(text)
        .into_iter()
        .map(|(code, _)| code)
        .collect()
}

/// Like `extract_blocks`, with each block's marker.
pub fn extract_marked_blocks(text: &str) -> Vec<(String, Marker)> {
    let mut blocks = Vec::new();
    let mut pos = 0;
    while let Some(rel) = text[pos..].find("```") {
        let open = pos + rel;
        let after = &text[open + 3..];
        let Some(nl) = after.find('\n') else {
            break;
        };
        let lang = after[..nl].trim();
        let body_start = &after[nl + 1..];
        let Some(close) = body_start.find("```") else {
            break;
        };
        if lang == "cp" || lang == "python" {
            let line_start = text[..open].rfind('\n').map_or(0, |i| i + 1);
            let indent = open - line_start;
            let code = dedent(&body_start[..close], indent);
            blocks.push((code, marker_above(&text[..open])));
        }
        pos = open + 3 + nl + 1 + close + 3;
    }
    blocks
}

/// Strips up to `indent` leading spaces from each line, as CommonMark does
/// for the body of a fence indented that far (a fence inside a list item).
fn dedent(code: &str, indent: usize) -> String {
    if indent == 0 {
        return code.to_string();
    }
    let mut out = String::with_capacity(code.len());
    for line in code.split_inclusive('\n') {
        let n = line.len() - line.trim_start_matches(' ').len();
        out.push_str(&line[n.min(indent)..]);
    }
    out.trim_end_matches(' ').to_string()
}

fn md_files_in(dir: &Path) -> Vec<std::path::PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<std::path::PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("md"))
        .collect();
    out.sort();
    out
}

/// Discovers fenced code blocks in `spec/core/*.md` and `spec/lib/*.md`.
pub fn discover(root: &Path, filter: Option<&str>) -> Vec<SpecBlock> {
    let spec_dir = root.join("spec");
    let mut paths = md_files_in(&spec_dir.join("core"));
    paths.extend(md_files_in(&spec_dir.join("lib")));

    let mut out = Vec::new();
    for path in paths {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let rel = path
            .strip_prefix(&spec_dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let mut seen: HashMap<String, usize> = HashMap::new();
        for (code, marker) in extract_marked_blocks(&text) {
            let base = format!("{rel}#{}", block_hash(&code));
            let occ = seen.entry(base.clone()).or_insert(0);
            *occ += 1;
            let id = if *occ == 1 {
                base
            } else {
                format!("{base}~{occ}")
            };
            if let Some(f) = filter {
                if !id.contains(f) && !code.contains(f) {
                    continue;
                }
            }
            out.push(SpecBlock { id, code, marker });
        }
    }
    out
}

fn eval_one(db: &mut Database, block: &SpecBlock) -> (String, (CaseOutcome, bool)) {
    let vpath = format!("<spec-block:{}>", block.id);
    let source = match &block.marker {
        Marker::Given(prelude) => format!("{prelude}\n{}", block.code),
        _ => block.code.clone(),
    };
    let (spec_diag, spec_failed) = runner::check_spec(db, &vpath, &source);
    let strict_ok = runner::compile_program(db, &vpath, &source).is_ok();
    if let (true, Marker::SpecOnly(reason)) = (spec_failed, &block.marker) {
        let skip = CaseOutcome {
            status: "skip".into(),
            detail: format!("spec-only: {reason}"),
        };
        return (block.id.clone(), (skip, strict_ok));
    }
    if let Marker::Rejects(code) = &block.marker {
        let hit = spec_failed && spec_diag.contains(&format!("[{code}]"));
        let outcome = CaseOutcome {
            status: if hit { "pass" } else { "error" }.into(),
            detail: if hit {
                String::new()
            } else if spec_failed {
                format!(
                    "wanted {code}; got {}",
                    runner::first_line_trimmed(&spec_diag, 140)
                )
            } else {
                format!("wanted {code}; got no diagnostic")
            },
        };
        return (block.id.clone(), (outcome, hit));
    }
    let outcome = if !spec_failed {
        CaseOutcome {
            status: "pass".into(),
            detail: String::new(),
        }
    } else {
        CaseOutcome {
            status: "error".into(),
            detail: runner::first_line_trimmed(&spec_diag, 160),
        }
    };
    (block.id.clone(), (outcome, strict_ok))
}

pub fn run(root: &Path, filter: Option<&str>) -> SuiteReport {
    let blocks = discover(root, filter);
    let results = runner::parallel_with_db(&blocks, eval_one);
    let mut outcomes = std::collections::BTreeMap::new();
    let mut strict_passed = 0usize;
    for (id, (outcome, strict_ok)) in results {
        if strict_ok {
            strict_passed += 1;
        }
        outcomes.insert(id, outcome);
    }
    let mut report = SuiteReport::from_outcomes(outcomes);
    report.strict_passed = Some(strict_passed);
    report
}
