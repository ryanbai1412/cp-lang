//! No source text crashes the front end, and every source gets the same
//! diagnostics from a warm database as from a fresh one. Every corpus `spec.cp` is
//! mutated with a fixed seed (truncated, a span cut out, a token or line
//! inserted, a line duplicated), and each copy goes through the checker,
//! lowering and the linter. A panic on any copy fails the test and prints
//! the seed and the mutated text, so the failure reproduces exactly.
//!
//! `CPL_FUZZ_ROUNDS=200` runs more copies per file than the default.
//!
//! Rule: `errors/R12`, handbook/guidelines/errors-checklist.md#r12-every-tool-shows-the-same-diagnostic.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};

#[path = "support/shards.rs"]
mod shards;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

const TOKENS: &[&str] = &[
    "(",
    ")",
    "[",
    "]",
    "{",
    "}",
    ":",
    "::",
    ",",
    ".",
    "..",
    "..=",
    "=",
    "==",
    "=>",
    "->",
    "?",
    "!",
    "??",
    "\"",
    "'",
    "`",
    "\\",
    "#",
    "@",
    "mut ",
    "let ",
    "var ",
    "fn ",
    "for ",
    "in ",
    "if ",
    "else",
    "match ",
    "return ",
    "lends ",
    "domain ",
    "struct ",
    "enum ",
    "use ",
    "with ",
    "0",
    "-1",
    "9223372036854775807",
    "1e9",
    "_",
    "x",
    "Int",
    "?Int",
    "[V]Int",
    "\n",
    "\n    ",
    "    ",
    "\t",
    "é",
    "//",
    "/*",
    "<",
    ">",
    "<<",
    "**",
    "%",
    "*",
    "+",
    "-",
];

/// A char boundary at or after `at`.
fn boundary(s: &str, mut at: usize) -> usize {
    at = at.min(s.len());
    while !s.is_char_boundary(at) {
        at += 1;
    }
    at
}

fn mutate(src: &str, rng: &mut Rng) -> String {
    let at = boundary(src, rng.below(src.len() + 1));
    match rng.below(5) {
        0 => src[..at].to_string(),
        1 => {
            let end = boundary(src, at + 1 + rng.below(40));
            format!("{}{}", &src[..at], &src[end..])
        }
        2 => {
            let tok = TOKENS[rng.below(TOKENS.len())];
            format!("{}{tok}{}", &src[..at], &src[at..])
        }
        3 => {
            let lines: Vec<&str> = src.lines().collect();
            let i = rng.below(lines.len());
            let j = rng.below(lines.len() + 1);
            let mut out = lines.clone();
            out.insert(j, lines.get(i).copied().unwrap_or(""));
            out.join("\n")
        }
        _ => {
            let a = boundary(src, rng.below(src.len() + 1));
            let b = boundary(src, a + rng.below(20));
            let piece = &src[a..b];
            format!("{}{piece}{}", &src[..at], &src[at..])
        }
    }
}

/// Every diagnostic the checker, lowering and linter report, as text.
fn front_end(db: &mut cpl_facade::Session, path: &str, src: &str) -> Vec<String> {
    *db = db.with_file(path, src);
    let lints = cpl_lint::lint(src, &cpl_lint::Config::default());
    db.diagnostics(path, cpl_facade::Layer::Built)
        .iter()
        .chain(&lints)
        .map(|d| {
            format!(
                "{:?} {}..{} {}",
                d.code, d.span.start, d.span.end, d.message
            )
        })
        .collect()
}

const SHARDS: usize = 8;
const COPY_SHARDS: usize = 4;

fn specs() -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = cpl_corpus::files(&root(), "spec.cp");
    paths.sort();
    paths
}

/// A spec's path under the repo root, which picks its shard.
fn key(path: &Path) -> String {
    path.strip_prefix(root()).unwrap().display().to_string()
}

fn keys() -> Vec<String> {
    specs().iter().map(|p| key(p)).collect()
}

fn mutated_programs_never_crash_and_report_deterministically(shard: usize) {
    let rounds: usize = cpl_env::FUZZ_ROUNDS.parse().unwrap_or(12);
    // Each spec keeps its place in the whole sorted corpus, which seeds it.
    let numbered: Vec<(usize, PathBuf)> = specs().into_iter().enumerate().collect();
    let paths = shards::mine(numbered, |(_, p)| key(p), shard, SHARDS);
    std::panic::set_hook(Box::new(|_| {}));
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let runs = paths.len() * rounds;
    let crashes: Vec<String> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|t| {
                let paths = &paths;
                scope.spawn(move || {
                    let mut crashes = Vec::new();
                    let mut db = cpl_facade::Session::default();
                    for &(k, ref path) in paths.iter().skip(t).step_by(threads) {
                        let src = std::fs::read_to_string(path).unwrap();
                        let name = path.to_string_lossy().into_owned();
                        for round in 0..rounds {
                            let seed = 0x9E37_79B9_7F4A_7C15 ^ ((k as u64) << 32 | round as u64);
                            let mut rng = Rng(seed | 1);
                            let mut copy = mutate(&src, &mut rng);
                            if rng.below(3) == 0 {
                                copy = mutate(&copy, &mut rng);
                            }
                            let ok =
                                catch_unwind(AssertUnwindSafe(|| front_end(&mut db, &name, &copy)));
                            match &ok {
                                Ok(warm) if round % 4 == 0 => {
                                    let fresh =
                                        front_end(&mut cpl_facade::Session::default(), &name, &copy);
                                    if *warm != fresh {
                                        crashes.push(format!(
                                            "{name} seed {seed:#x}: a warm database reports\n{}\nand a fresh one\n{}\n{copy}",
                                            warm.join("\n"),
                                            fresh.join("\n")
                                        ));
                                    }
                                }
                                _ => {}
                            }
                            if let Err(e) = ok {
                                let msg = e
                                    .downcast_ref::<String>()
                                    .cloned()
                                    .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                                    .unwrap_or_default();
                                crashes.push(format!("{name} seed {seed:#x}: {msg}\n{copy}"));
                                db = cpl_facade::Session::default();
                            }
                        }
                    }
                    crashes
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|w| w.join().unwrap())
            .collect()
    });
    let _ = std::panic::take_hook();
    println!("{runs} mutated programs, {} failures", crashes.len());
    assert!(
        crashes.is_empty(),
        "{} of {runs} mutated programs crash or differ; first:\n{}",
        crashes.len(),
        crashes
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n-----\n")
    );
}

shards::shard_tests!(
    mutated_programs_never_crash_and_report_deterministically,
    keys(),
    the_fuzz_shards_cover_every_spec,
    [
        mutated_programs_never_crash_and_report_deterministically_0 = 0,
        mutated_programs_never_crash_and_report_deterministically_1 = 1,
        mutated_programs_never_crash_and_report_deterministically_2 = 2,
        mutated_programs_never_crash_and_report_deterministically_3 = 3,
        mutated_programs_never_crash_and_report_deterministically_4 = 4,
        mutated_programs_never_crash_and_report_deterministically_5 = 5,
        mutated_programs_never_crash_and_report_deterministically_6 = 6,
        mutated_programs_never_crash_and_report_deterministically_7 = 7,
    ]
);

/// Diagnostics add under 10% to the front end: every corpus spec, broken by
/// a mutation that keeps its size (a token, a line or a piece pasted in) so
/// that it reports something, goes through the checker, lowering and linter
/// against a warm database, alternating with its clean original. Summed over
/// each of the corpus's path-hashed shards, the broken copies take at most
/// 1.10x the originals' time.
/// Rule: `errors/R17`, handbook/guidelines/errors-checklist.md#r17-a-mistake-costs-no-more-time-than-a-clean-file.
fn a_broken_copy_checks_within_10_percent_of_its_original(shard: usize) {
    use std::time::{Duration, Instant};
    let numbered: Vec<(usize, PathBuf)> = specs().into_iter().enumerate().collect();
    let paths = shards::mine(numbered, |(_, p)| key(p), shard, COPY_SHARDS);
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let (clean, broken, copies) = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|t| {
                let paths = &paths;
                scope.spawn(move || {
                    let mut db = cpl_facade::Session::default();
                    let (mut clean, mut broken, mut copies) = (Duration::ZERO, Duration::ZERO, 0);
                    for &(k, ref path) in paths.iter().skip(t).step_by(threads) {
                        let src = std::fs::read_to_string(path).unwrap();
                        let name = path.to_string_lossy().into_owned();
                        let mut rng = Rng((0x5EED_u64 << 32 | k as u64) | 1);
                        let quiet = front_end(&mut db, &name, &src).len();
                        let copy = (0..20).map(|_| mutate(&src, &mut rng)).find(|c| {
                            c.len() >= src.len() && front_end(&mut db, &name, c).len() > quiet
                        });
                        let Some(copy) = copy else { continue };
                        let time = |db: &mut cpl_facade::Session, text: &str| {
                            let start = Instant::now();
                            front_end(db, &name, text);
                            start.elapsed()
                        };
                        let (mut best_clean, mut best_broken) = (Duration::MAX, Duration::MAX);
                        for _ in 0..3 {
                            best_clean = best_clean.min(time(&mut db, &src));
                            best_broken = best_broken.min(time(&mut db, &copy));
                        }
                        clean += best_clean;
                        broken += best_broken;
                        copies += 1;
                    }
                    (clean, broken, copies)
                })
            })
            .collect();
        workers
            .into_iter()
            .map(|w| w.join().unwrap())
            .fold((Duration::ZERO, Duration::ZERO, 0), |a, b| {
                (a.0 + b.0, a.1 + b.1, a.2 + b.2)
            })
    });
    let ratio = broken.as_secs_f64() / clean.as_secs_f64();
    println!("{copies} broken copies: {broken:?} against {clean:?} clean, {ratio:.3}x");
    assert!(
        copies * 2 > paths.len(),
        "only {copies} of {} specs broke",
        paths.len()
    );
    assert!(
        ratio <= 1.10,
        "diagnostics add {:.0}% to the front end ({broken:?} against {clean:?})",
        (ratio - 1.0) * 100.0
    );
}

shards::shard_tests!(
    a_broken_copy_checks_within_10_percent_of_its_original,
    keys(),
    the_broken_copy_shards_cover_every_spec,
    [
        a_broken_copy_checks_within_10_percent_of_its_original_0 = 0,
        a_broken_copy_checks_within_10_percent_of_its_original_1 = 1,
        a_broken_copy_checks_within_10_percent_of_its_original_2 = 2,
        a_broken_copy_checks_within_10_percent_of_its_original_3 = 3,
    ]
);

/// One planted mistake: replace `src[start..end]` with `with`.
struct Edit {
    start: usize,
    end: usize,
    with: String,
}

type Sites = Box<dyn Fn(&str, &[Token]) -> Vec<Edit> + Sync>;

use cpl_syntax::{Token, TokenKind as K};

fn text<'a>(src: &'a str, t: &Token) -> &'a str {
    &src[t.span.start as usize..t.span.end as usize]
}

fn edit(start: u32, end: u32, with: impl Into<String>) -> Edit {
    Edit {
        start: start as usize,
        end: end as usize,
        with: with.into(),
    }
}

fn line_start(toks: &[Token], i: usize) -> bool {
    i == 0 || matches!(toks[i - 1].kind, K::Newline | K::Indent | K::Dedent)
}

/// Every token of `kind` becomes `with`.
fn swap(kind: K, with: &'static str) -> Sites {
    Box::new(move |_, toks| {
        toks.iter()
            .filter(|t| t.kind == kind)
            .map(|t| edit(t.span.start, t.span.end, with))
            .collect()
    })
}

/// A worker's runs, the mistakes it planted, its misses and its files.
type WorkerResult = (usize, Vec<usize>, Vec<(String, String)>, usize);

/// The end of the expression starting at token `i`: the last token before a
/// depth-0 `stop` token, a line end, or a closer of an enclosing bracket.
fn expr_end(toks: &[Token], i: usize, stop: &[K]) -> Option<usize> {
    let mut depth = 0i32;
    for (j, tok) in toks.iter().enumerate().skip(i) {
        match tok.kind {
            K::LParen | K::LBracket | K::LBrace => depth += 1,
            K::RParen | K::RBracket | K::RBrace if depth == 0 => return j.checked_sub(1),
            K::RParen | K::RBracket | K::RBrace => depth -= 1,
            K::Newline | K::Indent | K::Dedent | K::Eof | K::Comma if depth == 0 => {
                return j.checked_sub(1)
            }
            k if depth == 0 && stop.contains(&k) => return j.checked_sub(1),
            _ => {}
        }
    }
    None
}

/// Single-edit habits a contestant from C++, Python or Rust really writes:
/// the `fix` rows of `cpl_diag::habits` (a foreign word for a CP Lang one)
/// and a table of foreign shapes.
fn mistakes() -> Vec<(String, Sites)> {
    let mut out: Vec<(String, Sites)> = vec![
        ("`and` as `&&`".into(), swap(K::And, "&&")),
        ("`or` as `||`".into(), swap(K::Or, "||")),
        ("`==` as `===`".into(), swap(K::EqEq, "===")),
        (
            "`not x` as `!x`".into(),
            Box::new(|_, toks| {
                (0..toks.len().saturating_sub(1))
                    .filter(|&i| toks[i].kind == K::Not && toks[i + 1].kind != K::In)
                    .map(|i| edit(toks[i].span.start, toks[i + 1].span.start, "!"))
                    .collect()
            }),
        ),
        (
            "a `;` at a statement's end".into(),
            Box::new(|_, toks| {
                (1..toks.len())
                    .filter(|&i| toks[i].kind == K::Newline)
                    .filter(|&i| {
                        !matches!(
                            toks[i - 1].kind,
                            K::Colon | K::FatArrow | K::Newline | K::Indent | K::Dedent
                        )
                    })
                    .map(|i| edit(toks[i - 1].span.end, toks[i - 1].span.end, ";"))
                    .collect()
            }),
        ),
        (
            "`x += 1` as `x++`".into(),
            Box::new(|src, toks| {
                toks.windows(4)
                    .filter(|w| {
                        w[0].kind == K::Ident
                            && text(src, &w[1]) == "+="
                            && text(src, &w[2]) == "1"
                            && w[3].kind == K::Newline
                    })
                    .map(|w| edit(w[0].span.end, w[2].span.end, "++"))
                    .collect()
            }),
        ),
        (
            "`xs.len` as `xs.size()`".into(),
            Box::new(|src, toks| {
                toks.windows(3)
                    .filter(|w| {
                        w[0].kind == K::Dot && text(src, &w[1]) == "len" && w[2].kind != K::LParen
                    })
                    .map(|w| edit(w[1].span.start, w[1].span.end, "size()"))
                    .collect()
            }),
        ),
        (
            "`print(..)` as `println!(..)`".into(),
            Box::new(|src, toks| {
                toks.windows(2)
                    .filter(|w| text(src, &w[0]) == "print" && w[1].kind == K::LParen)
                    .map(|w| edit(w[0].span.start, w[0].span.end, "println!"))
                    .collect()
            }),
        ),
        (
            "`for x in xs:` as `for (x in xs):`".into(),
            Box::new(|src, toks| {
                let mut out = Vec::new();
                for i in 0..toks.len().saturating_sub(1) {
                    if toks[i].kind != K::For || !line_start(toks, i) {
                        continue;
                    }
                    let Some(last) = expr_end(toks, i + 1, &[K::Colon, K::If]) else {
                        continue;
                    };
                    if toks.get(last + 1).map(|t| t.kind) != Some(K::Colon) {
                        continue;
                    }
                    let (a, b) = (toks[i + 1].span.start, toks[last].span.end);
                    out.push(edit(a, b, format!("({})", &src[a as usize..b as usize])));
                }
                out
            }),
        ),
        (
            "Python's `a if c else b` for an inline `if`".into(),
            Box::new(|src, toks| {
                let mut out = Vec::new();
                let s = |a: usize, b: usize| {
                    &src[toks[a].span.start as usize..toks[b].span.end as usize]
                };
                for i in 1..toks.len() {
                    if toks[i].kind != K::If
                        || matches!(
                            toks[i - 1].kind,
                            K::Newline | K::Indent | K::Dedent | K::Colon | K::Else
                        )
                    {
                        continue;
                    }
                    let Some(c) = expr_end(toks, i + 1, &[K::Colon]) else {
                        continue;
                    };
                    let Some(a) = expr_end(toks, c + 2, &[K::Else, K::Colon, K::If]) else {
                        continue;
                    };
                    if toks.get(a + 1).map(|t| t.kind) != Some(K::Else)
                        || toks.get(a + 2).map(|t| t.kind) != Some(K::Colon)
                    {
                        continue;
                    }
                    let Some(b) = expr_end(toks, a + 3, &[K::Colon, K::If, K::Else]) else {
                        continue;
                    };
                    if b < a + 3 || toks[c + 1].kind != K::Colon || a < c + 2 || c < i + 1 {
                        continue;
                    }
                    let py = format!("{} if {} else {}", s(c + 2, a), s(i + 1, c), s(a + 3, b));
                    out.push(edit(toks[i].span.start, toks[b].span.end, py));
                }
                out
            }),
        ),
        (
            "C++'s `int x = 0` for `let x = 0`".into(),
            Box::new(|src, toks| {
                let mut out = Vec::new();
                for i in 0..toks.len().saturating_sub(4) {
                    let t = &toks[i..i + 4];
                    if !matches!(t[0].kind, K::Let | K::Var)
                        || t[1].kind != K::Ident
                        || !line_start(toks, i)
                    {
                        continue;
                    }
                    let name = text(src, &t[1]);
                    if t[2].kind == K::Eq {
                        out.push(edit(t[0].span.start, t[0].span.end, "int"));
                    } else if t[2].kind == K::Colon
                        && text(src, &t[3]) == "Int"
                        && toks.get(i + 4).map(|t| t.kind) == Some(K::Eq)
                    {
                        out.push(edit(t[0].span.start, t[3].span.end, format!("int {name}")));
                    }
                }
                out
            }),
        ),
        (
            "a `let` name typo'd at one use".into(),
            Box::new(typo_sites),
        ),
    ];
    for h in cpl_diag::habits::all().filter(|h| !h.fix.is_empty()) {
        let want: Vec<String> = cpl_syntax::lex(h.fix)
            .tokens
            .iter()
            .filter(|t| t.span.end > t.span.start)
            .map(|t| text(h.fix, t).to_string())
            .collect();
        let call = h.kind == cpl_diag::habits::Kind::Call;
        for name in h.names {
            let want = want.clone();
            let sites: Sites = Box::new(move |src, toks| {
                let n = want.len();
                (0..toks.len().saturating_sub(n))
                    .filter(|&i| {
                        (0..n).all(|j| text(src, &toks[i + j]) == want[j])
                            && toks[i..i + n]
                                .windows(2)
                                .all(|w| w[0].span.end == w[1].span.start)
                            && (i == 0 || toks[i - 1].kind != K::Dot)
                            && (toks[i + n].kind == K::LParen) == call
                            && toks[i + n].kind != K::FatArrow
                    })
                    .map(|i| edit(toks[i].span.start, toks[i + n - 1].span.end, *name))
                    .collect()
            });
            out.push((
                format!("habit `{}`: `{}` for `{}`", h.key, name, h.fix),
                sites,
            ));
        }
    }
    out
}

/// A later use of a `let`/`var` name, its last two letters swapped into a
/// name the file doesn't have.
fn typo_sites(src: &str, toks: &[Token]) -> Vec<Edit> {
    let names: std::collections::HashSet<&str> = toks
        .iter()
        .filter(|t| t.kind == K::Ident)
        .map(|t| text(src, t))
        .collect();
    let mut out = Vec::new();
    for i in 0..toks.len().saturating_sub(1) {
        if !matches!(toks[i].kind, K::Let | K::Var) || toks[i + 1].kind != K::Ident {
            continue;
        }
        let name = text(src, &toks[i + 1]);
        let b = name.as_bytes();
        if b.len() < 3 || !name.is_ascii() || b[b.len() - 1] == b[b.len() - 2] {
            continue;
        }
        let typo = format!(
            "{}{}{}",
            &name[..b.len() - 2],
            b[b.len() - 1] as char,
            b[b.len() - 2] as char
        );
        if names.contains(typo.as_str()) || cpl_syntax::KEYWORDS.contains(&typo.as_str()) {
            continue;
        }
        let uses = (i + 2..toks.len().saturating_sub(1)).filter(|&j| {
            text(src, &toks[j]) == name
                && !matches!(toks[j - 1].kind, K::Dot | K::Let | K::Var | K::For | K::Fn)
                && toks[j + 1].kind != K::Colon
        });
        out.extend(uses.map(|j| edit(toks[j].span.start, toks[j].span.end, typo.clone())));
    }
    out
}

/// Planted mistakes that still cascade, as `<spec dir> | <mistake>`; goal 0.
const CASCADES_KNOWN: &[&str] = &[];

/// One real mistake gives one error: every corpus spec that checks clean gets
/// one foreign habit planted at a seeded site (`mistakes`), and the checker
/// and lowering must report exactly one error, whose span touches the edit.
/// Warnings don't count. `CPL_FUZZ_ROUNDS` scales the sites per spec and
/// mistake (12 rounds is one site).
/// Rule: `errors/R01`, handbook/guidelines/errors-checklist.md.
#[test]
fn a_planted_mistake_gives_one_error() {
    let sites_per: usize = cpl_env::FUZZ_ROUNDS
        .parse::<usize>()
        .map_or(1, |r| r.div_ceil(12).max(1));
    let mut paths: Vec<PathBuf> = cpl_corpus::files(&root(), "spec.cp");
    paths.sort();
    let mistakes = mistakes();
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let errors = |db: &mut cpl_facade::Session, name: &str, src: &str| {
        *db = db.with_file(name, src);
        db.diagnostics(name, cpl_facade::Layer::Built)
            .into_iter()
            .filter(|d| d.severity == cpl_diag::Severity::Error)
            .collect::<Vec<_>>()
    };
    let results: Vec<WorkerResult> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|t| {
                let (paths, mistakes, errors) = (&paths, &mistakes, &errors);
                scope.spawn(move || {
                    let mut db = cpl_facade::Session::default();
                    let (mut runs, mut covered, mut bad, mut clean) = (0, Vec::new(), Vec::new(), 0);
                    for (k, path) in paths.iter().enumerate().skip(t).step_by(threads) {
                        let src = std::fs::read_to_string(path).unwrap();
                        let name = path.to_string_lossy().into_owned();
                        if !errors(&mut db, &name, &src).is_empty() {
                            continue;
                        }
                        clean += 1;
                        let dir = path
                            .parent()
                            .and_then(|p| p.file_name())
                            .map_or(String::new(), |d| d.to_string_lossy().into_owned());
                        let toks = cpl_syntax::lex(&src).tokens;
                        for (m, (what, sites)) in mistakes.iter().enumerate() {
                            let sites = sites(&src, &toks);
                            if sites.is_empty() {
                                continue;
                            }
                            covered.push(m);
                            for round in 0..sites_per.min(sites.len()) {
                                let seed = 0x0E_77_02u64 << 40 ^ (k as u64) << 24 ^ (m as u64) << 8 ^ round as u64;
                                let e = &sites[Rng(seed | 1).below(sites.len())];
                                let copy = format!("{}{}{}", &src[..e.start], e.with, &src[e.end..]);
                                let (a, b) = (e.start as u32, (e.start + e.with.len()) as u32);
                                let got = errors(&mut db, &name, &copy);
                                runs += 1;
                                let ok = got.len() == 1 && got[0].span.start <= b && got[0].span.end >= a;
                                if !ok {
                                    let shown: Vec<String> = got
                                        .iter()
                                        .map(|d| format!("  {:?} {}..{} {}", d.code, d.span.start, d.span.end, d.message))
                                        .collect();
                                    let line = copy[..e.start].matches('\n').count() + 1;
                                    bad.push((
                                        format!("{dir} | {what}"),
                                        format!(
                                            "{name} seed {seed:#x}: {what} at line {line} ({a}..{b}: `{}`), {} errors:\n{}",
                                            e.with,
                                            got.len(),
                                            shown.join("\n")
                                        ),
                                    ));
                                }
                            }
                        }
                    }
                    (runs, covered, bad, clean)
                })
            })
            .collect();
        workers.into_iter().map(|w| w.join().unwrap()).collect()
    });
    let runs: usize = results.iter().map(|r| r.0).sum();
    let covered: std::collections::BTreeSet<usize> =
        results.iter().flat_map(|r| r.1.iter().copied()).collect();
    let files: usize = results.iter().map(|r| r.3).sum();
    let bad: Vec<&(String, String)> = results.iter().flat_map(|r| &r.2).collect();
    let fresh: Vec<&&(String, String)> = bad
        .iter()
        .filter(|(key, _)| !CASCADES_KNOWN.contains(&key.as_str()))
        .collect();
    let fixed: Vec<&&str> = CASCADES_KNOWN
        .iter()
        .filter(|k| !bad.iter().any(|(key, _)| key == *k))
        .collect();
    println!(
        "{runs} planted mistakes ({} of {} kinds) in {files} of {} clean specs, {} give more or less than one error",
        covered.len(),
        mistakes.len(),
        paths.len(),
        bad.len()
    );
    for (m, (what, _)) in mistakes.iter().enumerate() {
        if !covered.contains(&m) {
            println!("never planted: {what}");
        }
    }
    assert!(
        fresh.is_empty() && fixed.is_empty(),
        "{} planted mistakes don't give exactly one error at the edit; {} known cascades now pass (delete them from CASCADES_KNOWN: {fixed:?}):\n{}",
        fresh.len(),
        fixed.len(),
        fresh.iter().map(|(_, msg)| msg.as_str()).collect::<Vec<_>>().join("\n-----\n")
    );
}
