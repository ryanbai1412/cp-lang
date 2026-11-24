//! Lints a library declares with `@lint(name, from: "...", to: "...")` on
//! one of its fns (R06.4.9). A program that uses the library, and the
//! library itself outside the fn, gets a finding where it writes `from`, with
//! `to` as its fix. The fn's parameter names are the holes.

use crate::walk::Visitor;
use crate::{File, Finding, Group, Level, Lint};
use cpl_check::Analysis;
use cpl_diag::{Code, Diagnostic, Span};
use cpl_syntax::ast::{Expr, Stmt};
use cpl_syntax::library_lint::{self, is_name, LibraryLint};
use cpl_syntax::{Token, TokenKind as K};
use std::collections::HashMap;
use std::sync::Mutex;

/// One `@lint` declaration a file can see.
pub(crate) struct Declared {
    pub lint: &'static Lint,
    decl: LibraryLint,
    /// `std.seq`; empty for the file itself.
    module: String,
    /// The tokens of each statement `from` spans; one for an expression.
    from: Vec<Vec<(K, String)>>,
    /// Names `from` binds itself (lambda parameters, loop and `let`/`var`
    /// variables).
    bound: Vec<String>,
}

/// The usable `@lint` declarations of `analysis`'s file and every module it uses.
pub(crate) fn collect(analysis: &Analysis) -> Vec<Declared> {
    let mut out = Vec::new();
    let mut add = |a: &Analysis, module: &str| {
        for item in &a.items {
            for stmt in &item.syntax.parsed.stmts {
                for decl in library_lint::declared(stmt, item.offset).0 {
                    let Some(from) = library_lint::run(&decl.from) else {
                        continue;
                    };
                    out.push(Declared {
                        lint: intern(&decl.name),
                        bound: bound_names(&from.concat()),
                        decl,
                        module: module.to_string(),
                        from,
                    });
                }
            }
        }
    };
    add(analysis, "");
    for m in analysis.modules() {
        add(&m.analysis, &m.name);
    }
    out
}

/// Whether a library this run has linted declares a lint called `name`.
pub(crate) fn is_declared(name: &str) -> bool {
    let all = ALL.lock().unwrap_or_else(|e| e.into_inner());
    all.iter().any(|l| l.name == name)
}

static ALL: Mutex<Vec<&'static Lint>> = Mutex::new(Vec::new());

/// The library lint named `name`, made once per name and kept: a finding
/// names its lint by a `&'static` reference.
pub(crate) fn intern(name: &str) -> &'static Lint {
    let mut all = ALL.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(lint) = all.iter().find(|l| l.name == name) {
        return lint;
    }
    let name: &'static str = Box::leak(name.to_string().into_boxed_str());
    let lint: &'static Lint = Box::leak(Box::new(Lint {
        code: Code(name),
        name,
        dir: "",
        default: Level::Warn,
        group: Group::Idiom,
        title: "a library's own lint",
        summary: "declared with `@lint` on a library fn",
        rule: Some("lints/R17"),
        replaces_whole: false,
        std_structure: false,
        quiet_in_namesake: false,
        no_fix: None,
        removed: None,
        family: None,
        practice: None,
        explanation: "",
        trigger: "",
        fix: "",
        near_miss: "",
    }));
    all.push(lint);
    lint
}

/// Names a pattern binds itself: the parameters before `=>`, and the
/// variable after `for`, `let` or `var`.
fn bound_names(toks: &[(K, String)]) -> Vec<String> {
    let mut out = Vec::new();
    for (i, (kind, _)) in toks.iter().enumerate() {
        if matches!(kind, K::For | K::Let | K::Var) {
            if let Some((K::Ident, t)) = toks.get(i + 1) {
                out.push(t.clone());
            }
        }
        if *kind != K::FatArrow || i == 0 {
            continue;
        }
        match &toks[i - 1] {
            (K::Ident, t) => out.push(t.clone()),
            (K::RParen, _) => {
                let mut j = i - 1;
                while j > 0 && toks[j].0 != K::LParen {
                    j -= 1;
                    if toks[j].0 == K::Ident && matches!(toks[j + 1].0, K::Comma | K::RParen) {
                        out.push(toks[j].1.clone());
                    }
                }
            }
            _ => {}
        }
    }
    out
}

/// Every expression and statement of the file, and the statements of each
/// block in order, in file positions.
#[derive(Default)]
struct Spans {
    base: u32,
    exprs: Vec<Span>,
    stmts: Vec<Span>,
    blocks: Vec<Vec<Span>>,
}

impl Visitor for Spans {
    fn stmts(&mut self, stmts: &[Stmt]) {
        let base = self.base;
        self.blocks
            .push(stmts.iter().map(|s| s.span.shift(base)).collect());
    }
    fn expr(&mut self, expr: &Expr) {
        self.exprs.push(expr.span.shift(self.base));
    }
    fn stmt(&mut self, stmt: &Stmt) {
        self.stmts.push(stmt.span.shift(self.base));
    }
}

pub(crate) fn find(file: &File, declared: &[Declared]) -> Vec<Finding> {
    if declared.is_empty() {
        return Vec::new();
    }
    let toks = library_lint::tokens(file.src);
    let text: Vec<&str> = toks
        .iter()
        .map(|t| &file.src[t.span.start as usize..t.span.end as usize])
        .collect();
    let kinds: Vec<K> = toks.iter().map(|t| t.kind).collect();
    let by_start: HashMap<u32, usize> = toks
        .iter()
        .enumerate()
        .map(|(i, t)| (t.span.start, i))
        .collect();
    let by_end: HashMap<u32, usize> = toks
        .iter()
        .enumerate()
        .map(|(i, t)| (t.span.end, i + 1))
        .collect();
    let mut spans = Spans::default();
    file.walk(&mut spans, |s, base| s.base = base);
    (spans.blocks).push(file.top_level().map(|(b, s)| s.span.shift(b)).collect());
    let mut ends: HashMap<u32, Vec<u32>> = HashMap::new();
    for e in &spans.exprs {
        ends.entry(e.start).or_default().push(e.end);
    }
    let mut candidates: Vec<(usize, usize, Span)> = spans
        .exprs
        .iter()
        .chain(&spans.stmts)
        .filter_map(|s| Some((*by_start.get(&s.start)?, *by_end.get(&s.end)?, *s)))
        .collect();
    candidates.sort_by_key(|c| (c.0, std::cmp::Reverse(c.1)));
    candidates.dedup_by_key(|c| (c.0, c.1));
    // A statement's span can run past its last token (a block's newline).
    let tok_range = |s: &Span| {
        let start = toks.partition_point(|t| t.span.start < s.start);
        let stop = toks.partition_point(|t| t.span.end <= s.end);
        (start < stop).then_some((start, stop))
    };
    let mut out = Vec::new();
    for d in declared {
        let all = d.from.concat();
        let repeated: Vec<bool> = (d.decl.holes.iter())
            .map(|h| all.iter().filter(|(_, t)| t == h).count() > 1)
            .collect();
        let matcher = |pat| Matcher {
            pat,
            holes: &d.decl.holes,
            bound: &d.bound,
            text: &text,
            kinds: &kinds,
            toks: &toks,
            ends: &ends,
            by_end: &by_end,
            repeated: repeated.clone(),
        };
        let parts: Vec<Matcher> = d.from.iter().map(|p| matcher(p.as_slice())).collect();
        // A run of statements matches statement by statement, in one block.
        let sites: Vec<Vec<Span>> = match d.from.len() {
            1 => candidates.iter().map(|c| vec![c.2]).collect(),
            k => {
                let mut runs: Vec<Vec<Span>> = (spans.blocks.iter())
                    .flat_map(|b| b.windows(k).map(<[Span]>::to_vec))
                    .collect();
                runs.sort_by_key(|r| (r[0].start, r[k - 1].end));
                runs.dedup();
                runs
            }
        };
        let mut taken: Vec<Span> = Vec::new();
        for site in sites {
            let span = Span::new(site[0].start, site[site.len() - 1].end);
            if (d.module.is_empty() && d.decl.item.contains(span))
                || taken.iter().any(|t| t.contains(span))
            {
                continue;
            }
            let mut holes = vec![None; d.decl.holes.len()];
            let mut names = Vec::new();
            let matched = site.iter().zip(&parts).all(|(s, m)| {
                tok_range(s)
                    .is_some_and(|(start, stop)| m.go(0, start, stop, &mut holes, &mut names))
            });
            if !matched {
                continue;
            }
            taken.push(span);
            let got: HashMap<&str, &str> = (d.decl.holes.iter().zip(&holes))
                .filter_map(|(h, run)| {
                    let (a, b) = (*run)?;
                    Some((
                        h.as_str(),
                        &file.src[toks[a].span.start as usize..toks[b - 1].span.end as usize],
                    ))
                })
                .collect();
            let fix = fill(&d.decl.to, &got, &names);
            let from = if d.module.is_empty() {
                "this file".to_string()
            } else {
                format!("`{}`", d.module)
            };
            let mut diag = Diagnostic::warning(d.lint.code, span, format!("write `{fix}`"))
                .with_help(format!(
                    "{from} declares this lint with `@lint` on `{}`",
                    d.decl.fn_name
                ));
            diag.fix(format!("use `{}`", d.decl.fn_name), span, fix);
            out.push(Finding { lint: d.lint, diag });
        }
    }
    out
}

struct Matcher<'a> {
    pat: &'a [(K, String)],
    holes: &'a [String],
    bound: &'a [String],
    text: &'a [&'a str],
    kinds: &'a [K],
    toks: &'a [Token],
    ends: &'a HashMap<u32, Vec<u32>>,
    by_end: &'a HashMap<u32, usize>,
    /// Per hole: whether `from` uses it twice, so it must be pure.
    repeated: Vec<bool>,
}

impl Matcher<'_> {
    /// Whether `pat[p..]` matches the tokens `j..stop` exactly.
    fn go(
        &self,
        p: usize,
        j: usize,
        stop: usize,
        holes: &mut Vec<Option<(usize, usize)>>,
        names: &mut Vec<(String, String)>,
    ) -> bool {
        if p == self.pat.len() {
            return j == stop;
        }
        if j >= stop {
            return false;
        }
        let (kind, text) = &self.pat[p];
        if is_name(*kind, text) {
            if let Some(h) = self.holes.iter().position(|x| x == text) {
                if let Some((a, b)) = holes[h] {
                    let n = b - a;
                    return j + n <= stop
                        && self.text[a..b] == self.text[j..j + n]
                        && self.go(p + 1, j + n, stop, holes, names);
                }
                for k in self.runs(j, stop) {
                    if self.repeated[h] && !pure(&self.kinds[j..k]) {
                        continue;
                    }
                    holes[h] = Some((j, k));
                    if self.go(p + 1, k, stop, holes, names) {
                        return true;
                    }
                    holes[h] = None;
                }
                return false;
            }
            if self.bound.contains(text) {
                if self.kinds[j] != K::Ident {
                    return false;
                }
                if let Some((_, user)) = names.iter().find(|(n, _)| n == text) {
                    return user == self.text[j] && self.go(p + 1, j + 1, stop, holes, names);
                }
                names.push((text.clone(), self.text[j].to_string()));
                if self.go(p + 1, j + 1, stop, holes, names) {
                    return true;
                }
                names.pop();
                return false;
            }
        }
        self.kinds[j] == *kind && self.text[j] == text && self.go(p + 1, j + 1, stop, holes, names)
    }

    /// The ends of the expressions starting at token `j` and ending by
    /// `stop`, a bracketed one with its brackets.
    fn runs(&self, j: usize, stop: usize) -> Vec<usize> {
        let start = self.toks[j].span.start;
        let mut out: Vec<usize> = (self.ends.get(&start).into_iter().flatten())
            .filter_map(|end| self.by_end.get(end).copied())
            .filter(|&k| k > j && k <= stop)
            .collect();
        if self.kinds[j] == K::LParen {
            let mut depth = 0;
            for (k, kind) in self.kinds.iter().enumerate().take(stop).skip(j) {
                match kind {
                    K::LParen | K::LBracket => depth += 1,
                    K::RParen | K::RBracket => depth -= 1,
                    _ => {}
                }
                if depth == 0 {
                    out.push(k + 1);
                    break;
                }
            }
        }
        out
    }
}

/// An expression a fix may write once where the code wrote it twice: it
/// calls nothing, so evaluating it twice or once gives the same program
/// (R06.4.9).
fn pure(kinds: &[K]) -> bool {
    !kinds
        .windows(2)
        .any(|w| w[1] == K::LParen && matches!(w[0], K::Ident | K::RParen | K::RBracket | K::Gt))
}

/// `to` with each hole replaced by what it matched, in brackets where the
/// code around it would otherwise bind tighter.
fn fill(to: &str, got: &HashMap<&str, &str>, names: &[(String, String)]) -> String {
    let toks = library_lint::tokens(to);
    let mut out = String::new();
    let mut last = 0;
    for (i, t) in toks.iter().enumerate() {
        let word = &to[t.span.start as usize..t.span.end as usize];
        let replacement = if let Some(m) = got.get(word).filter(|_| is_name(t.kind, word)) {
            let before = i.checked_sub(1).map(|i| toks[i].kind);
            let after = toks.get(i + 1).map(|t| t.kind);
            let delimited = matches!(before, Some(K::LParen | K::Comma) | None)
                && matches!(after, Some(K::RParen | K::Comma) | None);
            let inner = m
                .strip_prefix('(')
                .and_then(|m| m.strip_suffix(')'))
                .filter(|i| balanced(i));
            match inner {
                Some(inner) if delimited => inner.to_string(),
                _ if delimited || atomic(m) => m.to_string(),
                _ => format!("({m})"),
            }
        } else if let Some((_, user)) = names.iter().find(|(n, _)| n == word) {
            user.clone()
        } else {
            continue;
        };
        out.push_str(&to[last..t.span.start as usize]);
        out.push_str(&replacement);
        last = t.span.end as usize;
    }
    out.push_str(&to[last..]);
    out
}

fn balanced(s: &str) -> bool {
    let mut depth = 0i32;
    for c in s.chars() {
        depth += match c {
            '(' | '[' => 1,
            ')' | ']' => -1,
            _ => 0,
        };
        if depth < 0 {
            return false;
        }
    }
    depth == 0
}

/// No operator outside brackets: `xs[i].len`, `f(a + b)`, `3`.
fn atomic(src: &str) -> bool {
    let mut depth = 0;
    library_lint::tokens(src).iter().all(|t| {
        match t.kind {
            K::LParen | K::LBracket => depth += 1,
            K::RParen | K::RBracket => depth -= 1,
            _ => {}
        }
        depth > 0
            || matches!(
                t.kind,
                K::Ident
                    | K::SelfKw
                    | K::Int
                    | K::Float
                    | K::Str
                    | K::Dot
                    | K::RParen
                    | K::RBracket
                    | K::LParen
                    | K::LBracket
            )
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn brackets_balance_only_when_each_closes_one_opened_before_it() {
        assert!(super::balanced("f(x) + a[i]"));
        assert!(!super::balanced("a) + (b"));
        assert!(!super::balanced("(a"));
    }
}
