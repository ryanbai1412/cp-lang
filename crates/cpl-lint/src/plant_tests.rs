//! A lint catches the bug it names wherever it is written
//! (`handbook/guidelines/lints-checklist.md`, `lints/R25`): every record's
//! `before.cp`, with the names it binds renamed and planted after a clean
//! corpus program, fires its lint as often as alone; `near_miss.cp` planted
//! the same way stays quiet; and the host program gains no finding.

use super::*;
use crate::tests::all_on;
use cpl_syntax::{Token, TokenKind as K};
use std::collections::HashSet;

const BINDERS: [K; 9] = [
    K::Let,
    K::Var,
    K::For,
    K::Fn,
    K::Struct,
    K::Enum,
    K::Domain,
    K::Type,
    K::Const,
];

/// `src` with each name it binds (`let`, `var`, `for`, `fn`, `type`, ...)
/// renamed to `<name>2`, in code and in strings. Names also used as a
/// member (`xs.name`) keep their spelling.
fn rename(src: &str) -> String {
    let toks = cpl_syntax::lex(src).tokens;
    let text = |t: &Token| &src[t.span.start as usize..t.span.end as usize];
    let mut bound = HashSet::new();
    for (i, t) in toks.iter().enumerate() {
        if !BINDERS.contains(&t.kind) {
            continue;
        }
        let mut j = i + 1;
        while toks.get(j).is_some_and(|t| t.kind == K::Mut) {
            j += 1;
        }
        if t.kind == K::For {
            while toks
                .get(j)
                .is_some_and(|t| !matches!(t.kind, K::In | K::Colon))
            {
                if toks[j].kind == K::Ident {
                    bound.insert(text(&toks[j]).to_string());
                }
                j += 1;
            }
        } else if let Some(n) = toks.get(j).filter(|n| n.kind == K::Ident) {
            bound.insert(text(n).to_string());
        }
    }
    let dotted: HashSet<&str> = toks
        .windows(2)
        .filter(|w| w[0].kind == K::Dot && w[1].kind == K::Ident)
        .map(|w| text(&w[1]))
        .collect();
    bound.retain(|n| !n.starts_with('_') && !dotted.contains(n.as_str()));
    let mut out = String::new();
    let mut at = 0;
    for t in &toks {
        let piece = match t.kind {
            K::Ident if bound.contains(text(t)) => format!("{}2", text(t)),
            K::Str => rename_words(text(t), &bound),
            _ => continue,
        };
        out.push_str(&src[at..t.span.start as usize]);
        out.push_str(&piece);
        at = t.span.end as usize;
    }
    out.push_str(&src[at..]);
    out
}

fn rename_words(s: &str, bound: &HashSet<String>) -> String {
    let mut out = String::new();
    let mut word = String::new();
    for c in s.chars().map(Some).chain([None]) {
        if let Some(c) = c.filter(|c| c.is_alphanumeric() || *c == '_') {
            word.push(c);
            continue;
        }
        if bound.contains(&word) {
            word.push('2');
        }
        out.push_str(&word);
        word.clear();
        out.extend(c);
    }
    out
}

/// The three shortest corpus specs that lint clean.
fn hosts() -> Vec<String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/problems");
    let mut srcs: Vec<String> = std::fs::read_dir(root)
        .unwrap()
        .filter_map(|e| std::fs::read_to_string(e.unwrap().path().join("spec.cp")).ok())
        .filter(|s| s.ends_with('\n'))
        .collect();
    srcs.sort_by_key(|s| (s.len(), s.clone()));
    srcs.into_iter()
        .filter(|s| analyze_alone(s).is_some() && lint(s, &all_on()).is_empty())
        .take(3)
        .collect()
}

#[test]
fn a_record_fires_the_same_renamed_and_planted_in_a_clean_program() {
    let hosts = hosts();
    assert_eq!(hosts.len(), 3, "too few clean corpus specs to host records");
    let mut misses = Vec::new();
    for rec in LINTS {
        let code = rec.code.0;
        let found = |src: &str, from: usize| {
            lint(src, &all_on())
                .iter()
                .filter(|d| d.code.0 == code && d.span.start as usize >= from)
                .count()
        };
        let want = found(rec.trigger, 0);
        let before = rename(rec.trigger);
        let near = rename(rec.near_miss);
        let got = found(&before, 0);
        if got != want {
            misses.push(format!(
                "{}: renamed before.cp fires {got} of {want}\n{before}",
                rec.name
            ));
            continue;
        }
        if found(&near, 0) > 0 {
            misses.push(format!("{}: renamed near_miss.cp fires", rec.name));
            continue;
        }
        for host in &hosts {
            let from = host.len();
            let planted = format!("{host}{before}");
            let got = found(&planted, from);
            let stray: Vec<_> = lint(&planted, &all_on())
                .iter()
                .filter(|d| (d.span.start as usize) < from)
                .map(|d| d.code.0)
                .collect();
            if got != want || !stray.is_empty() || found(&format!("{host}{near}"), from) > 0 {
                misses.push(format!(
                    "{}: planted fires {got} of {want}, host gains {stray:?}\n{planted}",
                    rec.name
                ));
                break;
            }
        }
    }
    assert!(
        misses.is_empty(),
        "{} of {} records:\n{}",
        misses.len(),
        LINTS.len(),
        misses.join("\n")
    );
}
