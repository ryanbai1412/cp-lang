//! Every lint is a record under `spec/lints/<name>/`: `before.cp` fires,
//! `after.cp` fires nothing, `near_miss.cp` not it, a fix clears it, and the spec lists it.

use super::*;

use crate::tests::all_on;

fn codes(src: &str) -> Vec<&'static str> {
    lint(src, &all_on()).iter().map(|d| d.code.0).collect()
}

#[test]
fn every_lint_record_fires_on_before_only() {
    for lint in LINTS {
        let code = lint.code.0;
        assert!(
            codes(lint.trigger).contains(&code),
            "{}: before.cp does not fire",
            lint.name
        );
        assert!(
            codes(lint.fix).is_empty(),
            "{}: after.cp fires {:?}",
            lint.name,
            codes(lint.fix)
        );
        assert!(
            !codes(lint.near_miss).contains(&code),
            "{}: near_miss.cp fires",
            lint.name
        );
    }
}

#[test]
fn every_lint_record_directory_belongs_to_a_lint() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/lints");
    for entry in std::fs::read_dir(dir).unwrap() {
        let name = entry.unwrap().file_name().into_string().unwrap();
        assert!(
            lint_named(&name).is_some(),
            "spec/lints/{name} names no lint"
        );
    }
}

/// `Lnnnn name` pairs `spec/diagnostics.md` lists for the lints `cpl-lint`
/// owns (below L0600): its table rows `| L0201 | var_never_reassigned | .. |`
/// and the "Emitted by 0.2.x" paragraph, expanding `La-Lb prefix_{x, y}`
/// ranges there.
fn spec_lints() -> Vec<(u32, String)> {
    let doc = include_str!("../../../spec/diagnostics.md");
    let mut out: Vec<(u32, String)> = doc
        .lines()
        .filter_map(|row| {
            let cells: Vec<&str> = row.split('|').map(str::trim).collect();
            let code: u32 = cells.get(1)?.strip_prefix('L')?.parse().ok()?;
            (code < 600).then(|| (code, cells[2].to_string()))
        })
        .collect();
    let Some(start) = doc.find("Emitted by 0.2.x:") else {
        return out;
    };
    let para = &doc[start..];
    let para = &para[..para.find("\n\n").unwrap_or(para.len())];
    let text = para.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut rest = text.as_str();
    while let Some(at) = rest.find(" L") {
        rest = &rest[at + 1..];
        let code: String = rest[1..].chars().take_while(char::is_ascii_digit).collect();
        if code.len() != 4 {
            continue;
        }
        let lo: u32 = code.parse().unwrap();
        let after = &rest[5..];
        if let Some(range) = after.strip_prefix("-L") {
            let hi: u32 = range[..4].parse().unwrap();
            let names = &range[5..];
            let open = names.find('{').unwrap();
            let close = names.find('}').unwrap();
            let prefix = &names[..open];
            for (i, part) in names[open + 1..close].split(',').enumerate() {
                out.push((lo + i as u32, format!("{prefix}{}", part.trim())));
            }
            assert_eq!(
                out.last().unwrap().0,
                hi,
                "L{lo}-L{hi} names a different count"
            );
        } else {
            let name: String = after
                .trim_start()
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            out.push((lo, name));
        }
    }
    out
}

#[test]
fn every_lint_matches_the_spec_lint_list() {
    let spec = spec_lints();
    let ours: Vec<(u32, String)> = LINTS
        .iter()
        .map(|l| (l.code.0[1..].parse().unwrap(), l.name.to_string()))
        .collect();
    let missing: Vec<_> = ours.iter().filter(|e| !spec.contains(e)).collect();
    let extra: Vec<_> = spec.iter().filter(|e| !ours.contains(e)).collect();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "lints not in spec/diagnostics.md: {missing:?} (add a row `| Lnnnn | name | fires on |` \
         to its Lints section); listed there but not lints: {extra:?}"
    );
}

#[test]
fn every_record_fix_clears_its_lint_and_still_parses() {
    let config = all_on();
    for lint in LINTS {
        let (fixed, applied) = fix(lint.trigger, &config);
        if applied == 0 {
            continue;
        }
        assert!(
            cpl_syntax::parse_file(&fixed).diagnostics.is_empty(),
            "{}: fixing before.cp gives a file that does not parse:\n{fixed}",
            lint.name
        );
        assert!(
            !codes(&fixed).contains(&lint.code.0),
            "{}: fixing before.cp leaves the lint firing:\n{fixed}",
            lint.name
        );
    }
}

#[test]
fn every_record_after_passes_cpl_check() {
    for lint in LINTS {
        let errors = crate::fix_tests::checker_errors(lint.fix);
        assert!(errors.is_empty(), "{}: after.cp: {errors:?}", lint.name);
    }
}

/// Every `.cp` file under `dir`, recursively, with its text.
fn sources(dir: &std::path::Path, ext: &str, out: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).expect("readable dir").flatten() {
        let path = entry.path();
        if path.is_dir() {
            sources(&path, ext, out);
        } else if path.extension().is_some_and(|e| e == ext) {
            out.push(std::fs::read_to_string(&path).expect("readable file"));
        }
    }
}

fn is_ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// The names a lint's words may use: what `std` defines and the checker knows.
struct Vocabulary {
    std: Vec<String>,
    builtin: std::collections::HashSet<String>,
}

impl Vocabulary {
    fn load() -> Vocabulary {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut std = Vec::new();
        sources(&root.join("std"), "cp", &mut std);
        let mut rust = Vec::new();
        sources(&root.join("crates/cpl-check/src"), "rs", &mut rust);
        let builtin = rust
            .iter()
            .flat_map(|s| s.split('"').skip(1).step_by(2))
            .filter(|w| !w.is_empty() && w.chars().all(is_ident))
            .map(String::from)
            .collect();
        Vocabulary { std, builtin }
    }

    fn defines(&self, keyword: &str, name: &str, top_level: bool) -> bool {
        self.std
            .iter()
            .any(|s| Self::defined_in(s, keyword, name, top_level))
    }

    fn defined_in(src: &str, keyword: &str, name: &str, top_level: bool) -> bool {
        src.lines().any(|line| {
            let line = if top_level { line } else { line.trim_start() };
            let line = line.strip_prefix("pub ").unwrap_or(line);
            line.strip_prefix(keyword)
                .and_then(|rest| rest.strip_prefix(' '))
                .and_then(|rest| rest.strip_prefix(name))
                .is_some_and(|rest| !rest.starts_with(is_ident))
        })
    }

    fn is_type(&self, name: &str) -> bool {
        ["struct", "enum", "trait", "type", "const", "domain"]
            .iter()
            .any(|k| self.defines(k, name, false))
    }

    /// Why `name` in `span` names nothing, or `None` when it is real. `user` is
    /// the record the lint fired on, whose own names are fair to mention.
    fn missing(&self, span: &str, start: usize, name: &str, user: &str) -> Option<&'static str> {
        let end = start + name.len();
        let call = span[end..].starts_with('(');
        let receiver = span[..start].strip_suffix('.').map(|before| {
            let from = before.rfind(|c: char| !is_ident(c)).map_or(0, |i| i + 1);
            &before[from..]
        });
        let known = |n: &str| {
            self.builtin.contains(n) || user.split(|c: char| !is_ident(c)).any(|w| w == n)
        };
        let upper = name.starts_with(|c: char| c.is_ascii_uppercase());
        if name.len() == 1 || name.chars().all(|c| c.is_ascii_uppercase() || c == '_') {
            return None;
        }
        match receiver {
            Some("std") => {
                let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../std");
                (!root.join(format!("{name}.cp")).exists()).then_some("no such std module")
            }
            Some(ty) if ty.starts_with(|c: char| c.is_ascii_uppercase()) && self.is_type(ty) => {
                let homes: Vec<_> = self
                    .std
                    .iter()
                    .filter(|s| {
                        ["struct", "enum", "trait"]
                            .iter()
                            .any(|k| Self::defined_in(s, k, ty, true))
                    })
                    .collect();
                let found = homes.iter().any(|s| Self::defined_in(s, "fn", name, false));
                (!found && !known(name)).then_some("no such method on that type")
            }
            Some(_) => {
                (!self.defines("fn", name, false) && !known(name)).then_some("no such method")
            }
            None if upper => (!self.is_type(name) && !known(name)).then_some("no such type"),
            None if call => {
                (!self.defines("fn", name, true) && !known(name)).then_some("no such function")
            }
            None => None,
        }
    }
}

#[test]
fn every_help_names_real_vocabulary() {
    let vocab = Vocabulary::load();
    let mut bad = Vec::new();
    for lint in LINTS {
        for d in lint_all(lint.trigger) {
            let texts = std::iter::once(&d.message)
                .chain(&d.help)
                .chain(&d.notes)
                .chain(d.fixes.iter().map(|f| &f.title));
            for text in texts {
                for span in text.split('`').skip(1).step_by(2) {
                    let mut at = 0;
                    while let Some(off) =
                        span[at..].find(|c: char| c.is_ascii_alphabetic() || c == '_')
                    {
                        let start = at + off;
                        let len = span[start..]
                            .find(|c: char| !is_ident(c))
                            .unwrap_or(span.len() - start);
                        let name = &span[start..start + len];
                        let after_digit = span[..start].ends_with(|c: char| c.is_ascii_digit());
                        if !after_digit {
                            if let Some(why) = vocab.missing(span, start, name, lint.trigger) {
                                bad.push(format!("{}: `{name}` in `{span}`: {why}", lint.name));
                            }
                        }
                        at = start + len;
                    }
                }
            }
        }
    }
    assert!(
        bad.is_empty(),
        "lint words name things CPL lacks:\n{}",
        bad.join("\n")
    );
}

fn lint_all(src: &str) -> Vec<Diagnostic> {
    lint(src, &all_on())
}

#[test]
fn record_codes_are_unique() {
    let mut seen = std::collections::HashMap::new();
    for lint in LINTS {
        if let Some(other) = seen.insert(lint.code, lint.name) {
            panic!("{} is both `{other}` and `{}`", lint.code.0, lint.name);
        }
    }
}
