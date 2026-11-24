//! Lints a library declares with `@lint(name, from: "...", to: "...")` on
//! one of its fns (R06.4.9). A program that uses the library, and the
//! library itself outside the fn, gets a finding where it writes `from`, with
//! `to` as its fix. The fn's parameter names are the holes.

use crate::walk::Visitor;
use crate::{File, Finding, Group, Level, Lint};
use cpl_check::Analysis;
use cpl_diag::{Code, Span};
use cpl_syntax::ast::{Expr, ExprKind, Stmt, StmtKind, Type};
use cpl_syntax::library_lint::{self, is_name, Bound, LibraryLint};
use cpl_syntax::{Token, TokenKind as K};
use std::collections::HashMap;
use std::sync::Mutex;

/// One `@lint` declaration a file can see.
pub(crate) struct Declared {
    pub lint: &'static Lint,
    decl: LibraryLint,
    /// `std.seq`; empty for the file itself.
    module: String,
    /// The tokens of `from`, and where each statement it spans ends; one
    /// statement for an expression.
    from: Vec<(K, String)>,
    ends: Vec<usize>,
    /// Names `from` binds itself (lambda parameters, loop and `let`/`var`
    /// variables).
    bound: Vec<String>,
    /// Per token of `from`: a hole standing as a whole statement, which
    /// matches a run of statements.
    stmt_holes: Vec<bool>,
    /// The module's public names `to` uses, each as the linted file spells
    /// it (`sq` -> `geo.sq` under `use mylib.geo`).
    spell: Vec<(String, String)>,
}

/// The usable `@lint` declarations of `analysis`'s file and every module it uses.
pub(crate) fn collect(analysis: &Analysis) -> Vec<Declared> {
    let mut out = Vec::new();
    let mut add = |a: &Analysis, module: &str, spell: &dyn Fn(&str) -> Option<String>| {
        for item in &a.items {
            for stmt in &item.syntax.parsed.stmts {
                for decl in library_lint::declared(stmt, item.offset).0 {
                    let Some((from, ends)) = library_lint::run(&decl.from) else {
                        continue;
                    };
                    let names = library_lint::statement_names(&decl.from);
                    let stmt_holes = (0..from.len())
                        .map(|i| names.contains(&i) && decl.holes.contains(&from[i].1))
                        .collect();
                    let spell = (library_lint::tokens(&decl.to).iter())
                        .map(|t| &decl.to[t.span.start as usize..t.span.end as usize])
                        .filter_map(|w| Some((w.to_string(), spell(w)?)))
                        .collect();
                    out.push(Declared {
                        spell,
                        lint: intern(&decl.name),
                        bound: bound_names(&from),
                        stmt_holes,
                        decl,
                        module: module.to_string(),
                        from,
                        ends,
                    });
                }
            }
        }
    };
    add(analysis, "", &|_| None);
    for (i, m) in analysis.modules().iter().enumerate() {
        let public = |w: &str| {
            (m.analysis.items.iter())
                .flat_map(|item| &item.syntax.decls)
                .any(|d| d.public && d.name == w)
        };
        let spell = |w: &str| {
            if !public(w) {
                return None;
            }
            if let Some((local, _)) =
                (analysis.imports().iter()).find(|(_, imp)| imp.module == i && imp.name == w)
            {
                return (local != w).then(|| local.clone());
            }
            let (alias, _) = analysis.aliases().iter().find(|(_, &at)| at == i)?;
            Some(format!("{alias}.{w}"))
        };
        add(&m.analysis, &m.name, &spell);
    }
    out
}

/// Whether a library this run has linted declares a lint called `name`.
pub(crate) fn is_declared(name: &str) -> bool {
    let all = ALL.lock().unwrap_or_else(|e| e.into_inner());
    all.iter().any(|l| l.name == name)
}

static ALL: Mutex<Vec<&'static Lint>> = Mutex::new(Vec::new());

/// The library lint named `name`: the `spec/lints` record of that name when
/// there is one (std's idiom lints keep their codes, levels and
/// explanations), else one made once per name and kept, since a finding
/// names its lint by a `&'static` reference.
pub(crate) fn intern(name: &str) -> &'static Lint {
    if let Some(lint) = crate::lint_named(name) {
        return lint;
    }
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
        on: &[],
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

/// Names a pattern binds itself: the parameters before `=>`, the variables
/// after `for`, `let` or `var` (one name or a tuple of them), and a `fn` it
/// declares with its parameters.
fn bound_names(toks: &[(K, String)]) -> Vec<String> {
    let mut out = Vec::new();
    for (i, (kind, _)) in toks.iter().enumerate() {
        if matches!(kind, K::For | K::Let | K::Var | K::Fn) {
            let mut depth = 0;
            for (k, t) in &toks[i + 1..] {
                match k {
                    K::LParen => depth += 1,
                    K::RParen => depth -= 1,
                    K::Ident => out.push(t.clone()),
                    _ => {}
                }
                if depth <= 0 && *k != K::Comma {
                    break;
                }
            }
        }
        if *kind == K::Fn && toks.get(i + 2).is_some_and(|t| t.0 == K::LParen) {
            let mut depth = 0;
            for (j, (k, t)) in toks.iter().enumerate().skip(i + 2) {
                match k {
                    K::LParen | K::LBracket => depth += 1,
                    K::RParen | K::RBracket => depth -= 1,
                    K::Ident if depth == 1 && matches!(toks[j - 1].0, K::LParen | K::Comma) => {
                        out.push(t.clone())
                    }
                    _ => {}
                }
                if depth == 0 {
                    break;
                }
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
    /// Every type written, `C` in `Fenwick<C, Int>()`: a hole matches one
    /// as it matches an expression.
    types: Vec<Span>,
    blocks: Vec<Vec<Span>>,
    /// Where each assigned place starts: `from` cannot match a place, nor
    /// the base it indexes, since `to` is a call.
    places: Vec<u32>,
}

impl Visitor for Spans {
    fn stmts(&mut self, stmts: &[Stmt]) {
        let base = self.base;
        self.blocks
            .push(stmts.iter().map(|s| s.span.shift(base)).collect());
    }
    fn expr(&mut self, expr: &Expr) {
        self.exprs.push(expr.span.shift(self.base));
        if let ExprKind::Keep { target, .. } = &expr.kind {
            self.places.push(target.span.shift(self.base).start);
        }
    }
    fn ty(&mut self, ty: &Type) {
        self.types.push(ty.span.shift(self.base));
    }
    fn stmt(&mut self, stmt: &Stmt) {
        self.stmts.push(stmt.span.shift(self.base));
        if let StmtKind::Assign { target, .. } = &stmt.kind {
            self.places.push(target.span.shift(self.base).start);
        }
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
    for e in spans.exprs.iter().chain(&spans.types) {
        ends.entry(e.start).or_default().push(e.end);
    }
    let mut candidates: Vec<(usize, usize, Span)> = spans
        .exprs
        .iter()
        .filter(|e| !spans.places.contains(&e.start))
        .chain(&spans.stmts)
        .filter_map(|s| Some((*by_start.get(&s.start)?, *by_end.get(&s.end)?, *s)))
        .collect();
    cpl_diag::sort_by_key(&mut candidates, |c| {
        (c.0 as u64) << 32 | (u32::MAX as usize - c.1) as u64
    });
    candidates.dedup_by_key(|c| (c.0, c.1));
    // A statement's span can run past its last token (a block's newline).
    let before = |at: u32| toks.partition_point(|t| t.span.start < at);
    let tok_range = |s: &Span| {
        let (start, stop) = (before(s.start), before(s.end));
        (start < stop).then_some((start, stop))
    };
    // Where each run of consecutive statements of one block starts and ends,
    // in tokens: what a statement hole matches.
    let mut stmt_runs: HashMap<usize, Vec<usize>> = HashMap::new();
    if declared.iter().any(|d| d.stmt_holes.contains(&true)) {
        for block in &spans.blocks {
            let ranges: Vec<(usize, usize)> = block.iter().filter_map(tok_range).collect();
            for (i, &(start, _)) in ranges.iter().enumerate() {
                let runs = stmt_runs.entry(start).or_default();
                runs.extend(ranges[i..].iter().map(|r| r.1));
            }
        }
    }
    let mut out = Vec::new();
    // One fn's spellings of a lint share their sites: the first spelling
    // that matches a site takes it.
    let mut taken: HashMap<(&str, Span), Vec<Span>> = HashMap::new();
    for d in declared {
        let all = &d.from;
        let to = library_lint::tokens(&d.decl.to);
        let in_to = |h: &str| {
            to.iter()
                .any(|t| &d.decl.to[t.span.start as usize..t.span.end as usize] == h)
        };
        // R11: a compound `to` runs its right side before the place's indices.
        let compound = library_lint::code(&d.decl.to)
            .is_some_and(|to| library_lint::compound(&to, &d.decl.fn_name));
        let repeated: Vec<bool> = (d.decl.holes.iter())
            .map(|h| compound || all.iter().filter(|(_, t)| t == h).count() > 1 || !in_to(h))
            .collect();
        let matcher = |pat, stmt| Matcher {
            pat,
            stmt,
            stmt_runs: &stmt_runs,
            holes: &d.decl.holes,
            bound: &d.bound,
            text: &text,
            kinds: &kinds,
            toks: &toks,
            ends: &ends,
            by_end: &by_end,
            repeated: repeated.clone(),
        };
        let mut start = 0;
        let parts: Vec<Matcher> = (d.ends.iter())
            .map(|&end| {
                let at = std::mem::replace(&mut start, end)..end;
                matcher(&d.from[at.clone()], &d.stmt_holes[at])
            })
            .collect();
        // A run of statements matches statement by statement, in one block;
        // one statement matches as an expression or as a block's statement.
        let sites: Vec<Vec<Span>> = match d.ends.len() {
            1 => {
                let mut one: Vec<Vec<Span>> = (candidates.iter().map(|c| c.2))
                    .chain(spans.blocks.iter().flatten().copied())
                    .map(|s| vec![s])
                    .collect();
                cpl_diag::sort_by_key(&mut one, |r| r[0].order());
                one.dedup();
                one
            }
            k => {
                let mut runs: Vec<Vec<Span>> = (spans.blocks.iter())
                    .flat_map(|b| b.windows(k).map(<[Span]>::to_vec))
                    .collect();
                cpl_diag::sort_by_key(&mut runs, |r| Span::new(r[0].start, r[k - 1].end).order());
                runs.dedup();
                runs
            }
        };
        let taken = taken.entry((&d.decl.name, d.decl.item)).or_default();
        // A hole whose parameter is written as a scalar type matches only
        // code of that type, where the checker's types are at hand.
        let typed = |h: usize, (a, b): (usize, usize)| {
            let (Some(Some(want)), Some(facts)) = (d.decl.types.get(h), file.facts) else {
                return true;
            };
            let at = Span::new(toks[a].span.start, toks[b - 1].span.end);
            facts
                .type_of(at)
                .is_some_and(|t| t.widen().to_string() == *want)
        };
        for site in sites {
            // Up to the last token: a block statement's span runs on to the
            // next line's indent, which the fix must keep.
            let Some((_, stop)) = tok_range(&site[site.len() - 1]) else {
                continue;
            };
            let span = Span::new(site[0].start, toks[stop - 1].span.end);
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
            let bounded = |b: &Bound| {
                let at = d.decl.holes.iter().position(|h| *h == b.hole);
                at.and_then(|i| holes[i])
                    .is_some_and(|run| proven(file, &toks, &text, run, b))
            };
            if !matched
                || !d.decl.when.iter().all(bounded)
                || !(holes.iter().enumerate()).all(|(h, run)| run.is_none_or(|r| typed(h, r)))
            {
                continue;
            }
            taken.push(span);
            let first = before(span.start);
            let commented = (first..stop - 1).any(|i| {
                file.src[toks[i].span.end as usize..toks[i + 1].span.start as usize].contains("//")
            });
            let got: HashMap<&str, &str> = (d.decl.holes.iter().zip(&holes))
                .filter_map(|(h, run)| {
                    let (a, b) = (*run)?;
                    Some((
                        h.as_str(),
                        &file.src[toks[a].span.start as usize..toks[b - 1].span.end as usize],
                    ))
                })
                .collect();
            let column = |at: u32| {
                let line = file.src[..at as usize].rfind('\n').map_or(0, |i| i + 1);
                (line, at as usize - line)
            };
            let stmts: HashMap<&str, usize> = (d.from.iter().zip(&d.stmt_holes))
                .filter(|(_, s)| **s)
                .filter_map(|((_, h), _)| {
                    let at = d.decl.holes.iter().position(|x| x == h)?;
                    Some((h.as_str(), column(toks[holes[at]?.0].span.start).1))
                })
                .collect();
            let line = column(span.start).0;
            let indent: String = (file.src[line..].chars())
                .take_while(|c| *c == ' ' || *c == '\t')
                .collect();
            let fix = fill(&d.decl.to, &got, &names, &d.spell, &indent, &stmts);
            let subject = (to.iter())
                .map(|t| &d.decl.to[t.span.start as usize..t.span.end as usize])
                .find_map(|w| holes[d.decl.holes.iter().position(|h| h == w)?]);
            let from = if d.module.is_empty() {
                "this file".to_string()
            } else {
                format!("`{}`", d.module)
            };
            let mut diag = cpl_diag::msgs::lint_rules::LIBRARY_LINT_FIX
                .warning_as(d.lint.code, span, &[&fix])
                .with_help(format!(
                    "{from} declares this lint with `@lint` on `{}`",
                    d.decl.fn_name
                ));
            if let Some((a, b)) = subject {
                let at = Span::new(toks[a].span.start, toks[b - 1].span.end);
                diag = diag.with_label(at, format!("what `{}` takes", d.decl.fn_name));
            }
            if !commented && d.lint.no_fix.is_none() {
                diag.fix(format!("use `{}`", d.decl.fn_name), span, fix);
            }
            out.push(Finding { lint: d.lint, diag });
        }
    }
    out
}

struct Matcher<'a> {
    pat: &'a [(K, String)],
    /// Per token of `pat`: a hole that matches a run of statements.
    stmt: &'a [bool],
    stmt_runs: &'a HashMap<usize, Vec<usize>>,
    holes: &'a [String],
    bound: &'a [String],
    text: &'a [&'a str],
    kinds: &'a [K],
    toks: &'a [Token],
    ends: &'a HashMap<u32, Vec<u32>>,
    by_end: &'a HashMap<u32, usize>,
    /// Per hole: whether `from` uses it twice or `to` drops it, so it must
    /// be pure.
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
                let runs = if self.stmt[p] {
                    let runs = self.stmt_runs.get(&j).into_iter().flatten();
                    runs.copied().filter(|&k| k <= stop).collect()
                } else {
                    self.runs(j, stop)
                };
                for k in runs {
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

    /// The ends of the expressions and types starting at token `j` and
    /// ending by `stop`, a bracketed one with its brackets.
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

/// Whether the tokens `run`, what a hole matched, take only values in `b`'s
/// range: an integer literal in it, or code the checker gives a range type
/// whose ends lie in it (`k: 1..=10` for `k >= 1`).
fn proven(file: &File, toks: &[Token], text: &[&str], run: (usize, usize), b: &Bound) -> bool {
    let (a, z) = run;
    let literal = |t: &str| t.replace('_', "").parse::<i128>().ok();
    let (lo, hi) = match (&toks[a..z], &text[a..z]) {
        ([t], [v]) if t.kind == K::Int => (literal(v), literal(v)),
        ([m, t], [_, v]) if m.kind == K::Minus && t.kind == K::Int => {
            let v = literal(v).map(|v| -v);
            (v, v)
        }
        _ => {
            let span = Span::new(toks[a].span.start, toks[z - 1].span.end);
            match file.facts.and_then(|f| f.place_type(span)) {
                Some(cpl_check::Ty::Range { lo, hi, .. }) => (*lo, *hi),
                _ => (None, None),
            }
        }
    };
    b.lo.is_none_or(|want| lo.is_some_and(|lo| lo >= want))
        && b.hi.is_none_or(|want| hi.is_some_and(|hi| hi <= want))
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
/// code around it would otherwise bind tighter. Each line of `to` after the
/// first gets the site's `indent`, and a statement hole's lines move with
/// it: `stmts` gives the column its first line had.
fn fill(
    to: &str,
    got: &HashMap<&str, &str>,
    names: &[(String, String)],
    spell: &[(String, String)],
    indent: &str,
    stmts: &HashMap<&str, usize>,
) -> String {
    let toks = library_lint::tokens(to);
    let mut out = String::new();
    let mut last = 0;
    let lines = |s: &str| s.replace('\n', &format!("\n{indent}"));
    for (i, t) in toks.iter().enumerate() {
        let word = &to[t.span.start as usize..t.span.end as usize];
        let replacement = if let (Some(m), Some(&was)) = (got.get(word), stmts.get(word)) {
            let line = to[..t.span.start as usize].rfind('\n').map_or(0, |i| i + 1);
            let now = indent.len() + t.span.start as usize - line;
            let shift = |l: &str| {
                let kept = l.len() - l.trim_start_matches(' ').len();
                if l.trim().is_empty() {
                    String::new()
                } else {
                    " ".repeat(now) + &l[kept.min(was)..]
                }
            };
            let mut it = m.split('\n');
            let first = it.next().unwrap_or_default().to_string();
            it.fold(first, |acc, l| acc + "\n" + &shift(l))
        } else if let Some(m) = got.get(word).filter(|_| is_name(t.kind, word)) {
            let before = i.checked_sub(1).map(|i| toks[i].kind);
            let after = toks.get(i + 1).map(|t| t.kind);
            let delimited = (matches!(before, Some(K::LParen | K::Comma) | None)
                && matches!(after, Some(K::RParen | K::Comma) | None))
                || (before == Some(K::Eq) && after.is_none());
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
        } else if let Some((_, spelled)) = (spell.iter())
            .find(|(n, _)| n == word)
            .filter(|_| i == 0 || toks[i - 1].kind != K::Dot)
        {
            spelled.clone()
        } else {
            continue;
        };
        out.push_str(&lines(&to[last..t.span.start as usize]));
        out.push_str(&replacement);
        last = t.span.end as usize;
    }
    out.push_str(&lines(&to[last..]));
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
