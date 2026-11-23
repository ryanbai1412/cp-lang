use std::collections::HashMap;
use std::path::Path;

use cpl_check::Database;

use crate::runner;
use crate::scoreboard::{CaseOutcome, SuiteReport};

/// The section a heading names: its number (`### 1.3 Disjoint-set union` is `1.3`), else a
/// slug of its text.
fn section_label(heading: &str) -> String {
    let text = heading.trim_start_matches('#').trim();
    let first = text
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_end_matches('.');
    if !first.is_empty()
        && first
            .split('.')
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
    {
        return first.to_string();
    }
    let slug: String = text
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let slug = slug
        .split('-')
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        "top".into()
    } else {
        slug
    }
}

/// Stable case ids for a file's blocks: `rel#section:k`, the `k`th block under the heading
/// `section`. Editing a block keeps its id, so the ratchet sees it change, not disappear.
pub fn block_ids(rel: &str, text: &str) -> Vec<(String, String, Marker)> {
    let mut headings = Vec::new();
    let mut offset = 0;
    let mut fenced = false;
    for line in text.split_inclusive('\n') {
        let t = line.trim_start();
        if t.starts_with("```") {
            fenced = !fenced;
        } else if !fenced && line.starts_with('#') {
            headings.push((offset, section_label(line.trim_end())));
        }
        offset += line.len();
    }
    let mut count: HashMap<String, usize> = HashMap::new();
    scan_blocks(text)
        .into_iter()
        .map(|(open, code, marker)| {
            let at = headings.partition_point(|(o, _)| *o < open);
            let section = if at == 0 {
                "top".to_string()
            } else {
                headings[at - 1].1.clone()
            };
            let k = count.entry(section.clone()).or_insert(0);
            *k += 1;
            (format!("{rel}#{section}:{k}"), code, marker)
        })
        .collect()
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
/// `rejects` block passes only when checking it reports that code. A `given`
/// comment stacked right above a `rejects` one gives that block its setup.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Marker {
    #[default]
    None,
    SpecOnly(String),
    Given(String),
    Rejects(String),
    /// A `given:` comment stacked right above a `rejects:` one.
    GivenRejects(String, String),
}

fn comment_above(before: &str) -> Option<(&str, &str)> {
    let body = before
        .trim_end_matches([' ', '\t'])
        .strip_suffix('\n')?
        .trim_end_matches([' ', '\t'])
        .strip_suffix("-->")?;
    let open = body.rfind("<!--")?;
    Some((body[open + 4..].trim_start(), &body[..open]))
}

fn parse_marker(inner: &str) -> Marker {
    if let Some(r) = inner.strip_prefix("spec-only:") {
        Marker::SpecOnly(r.split_whitespace().collect::<Vec<_>>().join(" "))
    } else if let Some(c) = inner.strip_prefix("rejects:") {
        Marker::Rejects(c.trim().to_string())
    } else if let Some(c) = inner.strip_prefix("given:") {
        let lines: Vec<&str> = c.lines().filter(|l| !l.trim().is_empty()).collect();
        let indent = lines
            .iter()
            .map(|l| l.len() - l.trim_start().len())
            .min()
            .unwrap_or(0);
        let lines: Vec<&str> = lines.iter().map(|l| l[indent..].trim_end()).collect();
        Marker::Given(lines.join("\n"))
    } else {
        Marker::None
    }
}

fn marker_above(before: &str) -> Marker {
    let Some((inner, rest)) = comment_above(before) else {
        return Marker::None;
    };
    match parse_marker(inner) {
        Marker::Rejects(code) => match comment_above(rest).map(|(up, _)| parse_marker(up)) {
            Some(Marker::Given(given)) => Marker::GivenRejects(given, code),
            _ => Marker::Rejects(code),
        },
        m => m,
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
    scan_blocks(text)
        .into_iter()
        .map(|(_, code, m)| (code, m))
        .collect()
}

/// Each fenced block's opening offset, dedented body and marker.
fn scan_blocks(text: &str) -> Vec<(usize, String, Marker)> {
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
            blocks.push((open, code, marker_above(&text[..open])));
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
        for (id, code, marker) in block_ids(&rel, &text) {
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
        Marker::Given(prelude) | Marker::GivenRejects(prelude, _) => {
            format!("{prelude}\n{}", block.code)
        }
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
    if let Marker::Rejects(code) | Marker::GivenRejects(_, code) = &block.marker {
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
