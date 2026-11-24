use std::collections::HashMap;
use std::path::Path;

use cpl_check::Session;

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
/// The spec fences CP Lang as `cp` or `python`; the handbook also quotes real Python, so
/// under `handbook/` only `cp` blocks are CP Lang.
pub fn block_ids(rel: &str, text: &str) -> Vec<(String, String, Marker)> {
    let langs: &[&str] = if rel.starts_with("handbook/") {
        &["cp"]
    } else {
        SPEC_LANGS
    };
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
    scan_blocks(text, langs)
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

/// The comment ending on the line right above a fence; every CP block is a
/// test (handbook/guidelines/conformance-checklist.md, conformance/R7):
/// - none: the block checks clean as a program;
/// - `<!-- given: code -->`: `code` (top-level lines) is prepended, then it checks clean;
/// - `<!-- rejects: E0608 -->`: checking it reports that code (a `given:` stacked
///   right above gives it setup);
/// - `<!-- signatures: std/x.cp std/y.cp -->`: every declaration header in it is one std
///   declares in those files, once comments, `pub`, spacing and bodies are dropped;
/// - `<!-- excerpt: std/x.cp -->`: its lines appear in `std/x.cp` in order.
///
/// There is no escape hatch: a block that is not CP code takes a non-CP fence
/// language (`text`, `ebnf`, `cpp`, ...). `Invalid` is a block the gate rejects
/// as written (an untagged fence, `spec-only:`, a marker on a non-CP fence).
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Marker {
    #[default]
    None,
    Given(String),
    Rejects(String),
    /// A `given:` comment stacked right above a `rejects:` one.
    GivenRejects(String, String),
    Signatures(String),
    Excerpt(String),
    Invalid(String),
}

/// Fence languages that are not CP code, so their blocks are not tests.
pub const NON_CP_LANGS: &[&str] = &["cpp", "ebnf", "text", "toml", "sh", "console", "json"];

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
    let inner = inner.trim_end();
    if inner.starts_with("spec-only:") {
        Marker::Invalid(
            "`spec-only:` is gone: give the block a `given:` prelude, `signatures:`/`excerpt:` \
             against std, or a non-CP fence language"
                .into(),
        )
    } else if let Some(c) = inner.strip_prefix("rejects:") {
        Marker::Rejects(c.trim().to_string())
    } else if let Some(f) = inner.strip_prefix("signatures:") {
        Marker::Signatures(f.trim().to_string())
    } else if let Some(f) = inner.strip_prefix("excerpt:") {
        Marker::Excerpt(f.trim().to_string())
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
    scan_blocks(text, SPEC_LANGS)
        .into_iter()
        .map(|(_, code, m)| (code, m))
        .collect()
}

const SPEC_LANGS: &[&str] = &["cp", "python"];

/// Each CP block's opening offset, dedented body and marker. A fence with
/// no language, or one that is neither CP nor in `NON_CP_LANGS`, is a block
/// with an `Invalid` marker, as is a marker above a non-CP fence.
fn scan_blocks(text: &str, langs: &[&str]) -> Vec<(usize, String, Marker)> {
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
        let line_start = text[..open].rfind('\n').map_or(0, |i| i + 1);
        let indent = open - line_start;
        let code = || dedent(&body_start[..close], indent);
        let marker = marker_above(&text[..open]);
        if langs.contains(&lang) {
            blocks.push((open, code(), marker));
        } else if lang.is_empty() {
            let why = "a fence needs a language: `cp` for CP code, else one of NON_CP_LANGS";
            blocks.push((open, code(), Marker::Invalid(why.into())));
        } else if !NON_CP_LANGS.contains(&lang) && !SPEC_LANGS.contains(&lang) {
            let why = format!("fence language `{lang}` is neither CP nor one of NON_CP_LANGS");
            blocks.push((open, code(), Marker::Invalid(why)));
        } else if marker != Marker::None {
            let why = format!("a test marker sits above a `{lang}` fence, which is not CP code");
            blocks.push((open, code(), Marker::Invalid(why)));
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

/// Discovers fenced code blocks in `spec/{core,lib,tools}/*.md` and the handbook's
/// language docs, `handbook/language/*.md`, which must say what the spec says
/// (handbook/guidelines/spec-rules-checklist.md, spec-rules/R6).
pub fn discover(root: &Path, filter: Option<&str>) -> Vec<SpecBlock> {
    let spec_dir = root.join("spec");
    let mut paths: Vec<(std::path::PathBuf, &Path)> = Vec::new();
    for dir in ["core", "lib", "tools"] {
        paths.extend(
            md_files_in(&spec_dir.join(dir))
                .into_iter()
                .map(|p| (p, spec_dir.as_path())),
        );
    }
    paths.extend(
        md_files_in(&root.join("handbook/language"))
            .into_iter()
            .map(|p| (p, root)),
    );

    let mut out = Vec::new();
    for (path, base) in paths {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let rel = path
            .strip_prefix(base)
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

fn outcome(failure: Option<String>) -> CaseOutcome {
    match failure {
        None => CaseOutcome {
            status: "pass".into(),
            detail: String::new(),
        },
        Some(detail) => CaseOutcome {
            status: "error".into(),
            detail,
        },
    }
}

fn eval_one(root: &Path, db: &mut Session, block: &SpecBlock) -> (String, (CaseOutcome, bool)) {
    let id = block.id.clone();
    let source = match &block.marker {
        Marker::Invalid(why) => return (id, (outcome(Some(why.clone())), false)),
        Marker::Signatures(file) => {
            let failure = against_std(root, file, |std| signatures_unmet(&block.code, std, file));
            let ok = failure.is_none();
            return (id, (outcome(failure), ok));
        }
        Marker::Excerpt(file) => {
            let failure = against_std(root, file, |std| excerpt_unmet(&block.code, std));
            let ok = failure.is_none();
            return (id, (outcome(failure), ok));
        }
        Marker::Given(prelude) | Marker::GivenRejects(prelude, _) => {
            format!("{prelude}\n{}", block.code)
        }
        _ => block.code.clone(),
    };
    // A real path under `spec/core/lib-root`, whose `cp.toml` names the
    // `lib` library, so blocks can show `use lib.x`.
    let name: String = block
        .id
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    let vpath = root
        .join("spec/core/lib-root")
        .join(format!("{name}.cp"))
        .display()
        .to_string();
    let (spec_diag, spec_failed) = runner::check_spec(db, &vpath, &source);
    if let Marker::Rejects(code) | Marker::GivenRejects(_, code) = &block.marker {
        let hit = spec_failed && spec_diag.contains(&format!("[{code}]"));
        let failure = if hit {
            None
        } else if spec_failed {
            Some(format!(
                "wanted {code}; got {}",
                runner::first_line_trimmed(&spec_diag, 140)
            ))
        } else {
            Some(format!("wanted {code}; got no diagnostic"))
        };
        return (id, (outcome(failure), hit));
    }
    let strict_ok = runner::compile_program(db, &vpath, &source).is_ok();
    let failure = spec_failed.then(|| runner::first_line_trimmed(&spec_diag, 160));
    (id, (outcome(failure), strict_ok))
}

/// The text of `std/x.cp`, or why a marker naming it is wrong.
/// The text of the std files a marker names, space-separated, joined.
fn std_text(root: &Path, files: &str) -> Result<String, String> {
    let mut text = String::new();
    for file in files.split_whitespace() {
        if !file.starts_with("std/") || !file.ends_with(".cp") {
            return Err(format!(
                "`{file}`: a marker names std files as `std/x.cp std/y.cp`"
            ));
        }
        text += &std::fs::read_to_string(root.join(file)).map_err(|e| format!("`{file}`: {e}"))?;
        text.push('\n');
    }
    if text.is_empty() {
        return Err("the marker names no std file".into());
    }
    Ok(text)
}

fn against_std(
    root: &Path,
    file: &str,
    unmet: impl FnOnce(&str) -> Option<String>,
) -> Option<String> {
    match std_text(root, file) {
        Ok(text) => unmet(&text),
        Err(why) => Some(why),
    }
}

/// A line without its `//` comment, its whitespace collapsed.
fn code_of(line: &str) -> String {
    let line = line.trim_start();
    let code = line.find("//").map_or(line, |i| &line[..i]);
    code.split_whitespace().collect::<Vec<_>>().join(" ")
}

const DECL_HEADS: &[&str] = &[
    "fn ", "struct ", "enum ", "trait ", "type ", "context ", "law ",
];

/// A declaration header, normalized: no decorators' `pub`, no body (`:` at
/// the end or ` = ...`), whitespace collapsed; `None` for other lines.
pub fn decl_header(code: &str) -> Option<String> {
    let words: Vec<&str> = code
        .split(' ')
        .filter(|w| *w != "pub" && *w != "extern")
        .collect();
    let head = words.join(" ");
    let start = head.find(|c: char| c != '@').map(|_| {
        let mut rest = head.as_str();
        while rest.starts_with('@') {
            rest = rest.split_once(' ').map_or("", |(_, r)| r);
        }
        rest
    })?;
    if !DECL_HEADS.iter().any(|h| start.starts_with(h)) {
        return None;
    }
    let mut depth = 0i32;
    let mut end = head.len();
    for (i, c) in head.char_indices() {
        match c {
            '(' | '<' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            '>' if !head[..i].ends_with('-') => depth -= 1,
            '=' if depth == 0 && head[i..].starts_with("= ") && head[..i].ends_with(' ') => {
                end = i;
                break;
            }
            _ => {}
        }
    }
    let head = head[..end].trim_end();
    let head = head
        .strip_suffix(" =")
        .unwrap_or(head)
        .trim_end_matches(':')
        .trim_end();
    Some(head.to_string())
}

/// The declaration headers in `code`, with lines that continue an open
/// bracket or start with `->` joined to the line before.
fn decl_headers(code: &str) -> Vec<String> {
    let mut joined: Vec<String> = Vec::new();
    let mut open = 0i32;
    for line in code.lines() {
        let c = code_of(line);
        if c.is_empty() {
            continue;
        }
        let continues = open > 0 || c.starts_with("->");
        match joined.last_mut() {
            Some(last) if continues => {
                if !last.ends_with('(') {
                    last.push(' ');
                }
                last.push_str(&c);
            }
            _ => joined.push(c.clone()),
        }
        open += c.matches(['(', '[']).count() as i32 - c.matches([')', ']']).count() as i32;
    }
    joined.iter().filter_map(|l| decl_header(l)).collect()
}

/// A header with spacing and trailing commas gone, so reflowing std never
/// breaks a spec block.
fn header_key(h: &str) -> String {
    let h = h.replace("(pub ", "(").replace(", pub ", ", ");
    let k: String = h.chars().filter(|c| !c.is_whitespace()).collect();
    k.replace(",)", ")").replace(",>", ">").replace(",]", "]")
}

/// The name a header declares: the word after its keyword.
fn header_name(h: &str) -> &str {
    let mut h = h;
    while h.starts_with('@') {
        h = h.split_once(' ').map_or("", |(_, r)| r);
    }
    let rest = h.split_once(' ').map_or("", |(_, r)| r);
    let end = rest
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(rest.len());
    &rest[..end]
}

/// A `fn m.name` header with `m` one of the `files`' modules, as its module
/// declares it: `fn name`. A free fn whose name a prelude fn already takes is
/// reached qualified, so the spec spells it that way.
fn as_declared(h: &str, files: &str) -> String {
    files
        .split_whitespace()
        .filter_map(|f| f.strip_prefix("std/")?.strip_suffix(".cp"))
        .find_map(|m| Some(format!("fn {}", h.strip_prefix(&format!("fn {m}."))?)))
        .unwrap_or_else(|| h.to_string())
}

/// Why a `signatures:` block disagrees with the std `files`, or `None`.
pub fn signatures_unmet(block: &str, std: &str, files: &str) -> Option<String> {
    let std_headers = decl_headers(std);
    let have: std::collections::HashSet<String> =
        std_headers.iter().map(|h| header_key(h)).collect();
    let wanted: Vec<String> = decl_headers(block)
        .iter()
        .map(|h| as_declared(h, files))
        .collect();
    if wanted.is_empty() {
        return Some("a `signatures:` block declares nothing".into());
    }
    let missing: Vec<&String> = wanted
        .iter()
        .filter(|h| !have.contains(&header_key(h)))
        .collect();
    if missing.is_empty() {
        return None;
    }
    let lines: Vec<String> = missing
        .iter()
        .map(|h| {
            let std_says = std_headers
                .iter()
                .find(|s| header_name(s) == header_name(h))
                .map_or_else(
                    || "no declaration of that name".to_string(),
                    |s| format!("std has `{s}`"),
                );
            format!("\n    `{h}` ({std_says})")
        })
        .collect();
    Some(format!(
        "{} of {} declarations are not in std as written:{}",
        missing.len(),
        wanted.len(),
        lines.concat(),
    ))
}

/// Why an `excerpt:` block is not a run of lines of std, or `None`.
pub fn excerpt_unmet(block: &str, std: &str) -> Option<String> {
    let mut std_lines = std.lines().map(code_of).filter(|l| !l.is_empty());
    let wanted: Vec<String> = block
        .lines()
        .map(code_of)
        .filter(|l| !l.is_empty())
        .collect();
    if wanted.is_empty() {
        return Some("an `excerpt:` block has no code".into());
    }
    for w in &wanted {
        if !std_lines.any(|l| &l == w) {
            return Some(format!("`{w}` is not in std, in this order"));
        }
    }
    None
}

pub fn run(root: &Path, filter: Option<&str>) -> SuiteReport {
    let blocks = discover(root, filter);
    let results = runner::parallel_with_db(&blocks, |db, b| eval_one(root, db, b));
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
