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
}

/// Extracts fenced ```cp / ```python code blocks from markdown text.
pub fn extract_blocks(text: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find("```") {
        let after = &rest[open + 3..];
        let Some(nl) = after.find('\n') else {
            break;
        };
        let lang = after[..nl].trim();
        let body_start = &after[nl + 1..];
        let Some(close) = body_start.find("```") else {
            break;
        };
        if lang == "cp" || lang == "python" {
            blocks.push(body_start[..close].to_string());
        }
        rest = &body_start[close + 3..];
    }
    blocks
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
        for code in extract_blocks(&text) {
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
            out.push(SpecBlock { id, code });
        }
    }
    out
}

fn eval_one(db: &mut Database, block: &SpecBlock) -> (String, (CaseOutcome, bool)) {
    let vpath = format!("<spec-block:{}>", block.id);
    let (spec_diag, spec_failed) = runner::check_spec(db, &vpath, &block.code);
    let strict_ok = runner::compile_program(db, &vpath, &block.code).is_ok();
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
