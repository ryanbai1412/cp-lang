//! Every lint is a record under `spec/lints/<name>/`: `before.cp` fires,
//! `after.cp` fires nothing, `near_miss.cp` not it, a fix clears it, and the spec lists it.

use super::*;

fn codes(src: &str) -> Vec<&'static str> {
    lint(src, &Config::default())
        .iter()
        .map(|d| d.code.0)
        .collect()
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
    let config = Config::default();
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
