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
use std::sync::{Mutex, OnceLock};

/// One `@lint` declaration a file can see.
#[derive(Clone)]
pub(crate) struct Declared {
    pub lint: &'static Lint,
    decl: LibraryLint,
    /// `std.seq`; empty for the file itself.
    module: String,
    /// The tokens of `from`, and where each statement it spans ends; one
    /// statement for an expression.
    from: Vec<(K, String)>,
    ends: Vec<usize>,
    /// The tokens of each `also:` expression.
    also: Vec<Vec<(K, String)>>,
    /// Names `from` binds itself (lambda parameters, loop and `let`/`var`
    /// variables).
    bound: Vec<String>,
    /// Per token of `from`: a hole standing as a whole statement, which
    /// matches a run of statements.
    stmt_holes: Vec<bool>,
    /// Per token of `from`, whether it is a named argument's label.
    labels: Vec<bool>,
    /// The first hole `from` calls a method on: the finding labels it when
    /// `to` writes no hole, so a record's `on` types check the receiver.
    receiver: Option<usize>,
    /// Names `from` declares with a written type and `to` declares again
    /// without one: the fix may change their type.
    rebinds: Vec<String>,
    /// `reads:`: the read `from`'s name has later, and what the fix writes
    /// for each.
    reads: Option<(Vec<(K, String)>, String)>,
    /// Per name `to` declares, the holes its initializer uses.
    inits: Vec<(String, Vec<String>)>,
    /// The module's public names `to` uses, each as the linted file spells
    /// it (`sq` -> `geo.sq` under `use mylib.geo`).
    spell: Vec<(String, String)>,
}

/// The usable `@lint` declarations of `analysis`'s file, every module it
/// uses, and std's advice.
pub(crate) fn collect(analysis: &Analysis) -> Vec<Declared> {
    let mut out = Vec::new();
    let mut add = |a: &Analysis, module: &str, spell: &dyn Fn(&str) -> Option<String>| {
        for item in &a.items {
            for stmt in &item.syntax.parsed.stmts {
                for decl in library_lint::declared(stmt, item.offset).0 {
                    out.extend(declare(decl, module, spell));
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
    let loaded = |name: &str| analysis.modules().iter().any(|m| m.name == name);
    for (module, decl) in std_lints().iter().filter(|(m, _)| !loaded(m)) {
        out.extend(declare(decl.clone(), module, &|_| None));
    }
    out
}

/// What `collect` keeps of `decl`: none when its `from` is no run of code.
fn declare(
    decl: LibraryLint,
    module: &str,
    spell: &dyn Fn(&str) -> Option<String>,
) -> Option<Declared> {
    let (from, ends) = library_lint::run(&decl.from)?;
    let names = library_lint::statement_names(&decl.from);
    let stmt_holes = (0..from.len())
        .map(|i| names.contains(&i) && decl.holes.contains(&from[i].1))
        .collect();
    let spell = (library_lint::tokens(&decl.to).iter())
        .map(|t| &decl.to[t.span.start as usize..t.span.end as usize])
        .filter_map(|w| Some((w.to_string(), spell(w)?)))
        .collect();
    let labels = library_lint::labels(&from.iter().map(|t| t.0).collect::<Vec<_>>());
    let receiver = (0..from.len().saturating_sub(1))
        .filter(|&i| !labels[i] && from[i + 1].0 == K::Dot)
        .find_map(|i| decl.holes.iter().position(|h| *h == from[i].1));
    let declared_in = |toks: &[(K, String)], typed: bool| -> Vec<String> {
        (toks.windows(3))
            .filter(|w| matches!(w[0].0, K::Let | K::Var) && w[1].0 == K::Ident)
            .filter(|w| (w[2].0 == K::Colon) == typed)
            .map(|w| w[1].1.clone())
            .collect()
    };
    let to_toks = library_lint::run(&decl.to).map_or(Vec::new(), |r| r.0);
    let to_names = declared_in(&to_toks, false);
    let rebinds = (declared_in(&from, true).into_iter())
        .filter(|n| to_names.contains(n))
        .collect();
    let reads = (decl.reads.split_once("->"))
        .and_then(|(r, w)| Some((library_lint::code(r.trim())?, w.trim().to_string())));
    let (to_run, to_ends) = library_lint::run(&decl.to).unwrap_or_default();
    let mut inits = Vec::new();
    let mut start = 0;
    for end in to_ends {
        let stmt = &to_run[std::mem::replace(&mut start, end)..end];
        if let ([(K::Let | K::Var, _), (K::Ident, n), ..], Some(eq)) =
            (stmt, stmt.iter().position(|t| t.0 == K::Eq))
        {
            let hs = (stmt[eq + 1..].iter())
                .filter(|t| decl.holes.contains(&t.1))
                .map(|t| t.1.clone());
            inits.push((n.clone(), hs.collect()));
        }
    }
    let mut also = Vec::new();
    for line in decl.also.lines().filter(|l| !l.trim().is_empty()) {
        also.push(library_lint::code(line)?);
    }
    Some(Declared {
        also,
        spell,
        lint: intern(&decl.name),
        bound: bound_names(&from),
        stmt_holes,
        labels,
        receiver,
        rebinds,
        reads,
        inits,
        decl,
        module: module.to_string(),
        from,
        ends,
    })
}

/// Every std module's `@lint`s, read once from std's outline: a program
/// that hand-writes what a std fn does seldom loads its module, and std's
/// names need no `use`.
pub(crate) fn std_lints() -> &'static [(String, LibraryLint)] {
    static ALL: OnceLock<Vec<(String, LibraryLint)>> = OnceLock::new();
    ALL.get_or_init(|| {
        let mut out = Vec::new();
        for (name, outline) in cpl_check::std_index() {
            if !outline.contains("@lint(") {
                continue;
            }
            let module = format!("std.{}", name.replace('/', "."));
            for (offset, item) in cpl_syntax::parse_file(outline).items {
                for stmt in &item.stmts {
                    let decls = library_lint::declared(stmt, offset).0;
                    out.extend(decls.into_iter().map(|d| (module.clone(), d)));
                }
            }
        }
        out
    })
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

/// Every expression, type and statement of the file, and the statements of
/// each block in order, in file positions.
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
    /// Every `let`, `var` or `for` pattern, `(u, v)` in `let (u, v) = e`:
    /// a hole matches one as it matches a type.
    pats: Vec<Span>,
    /// The value of each constant expression (`-1`, `10**18`), by its span.
    consts: HashMap<(u32, u32), i128>,
    /// Every fn the file declares, by name: a library fn's lint with a fix
    /// keeps out of the program's own fn of its fn's name, which the fix
    /// would make call itself.
    fns: Vec<(String, Span)>,
}

impl Visitor for Spans {
    fn stmts(&mut self, stmts: &[Stmt]) {
        let base = self.base;
        self.blocks
            .push(stmts.iter().map(|s| s.span.shift(base)).collect());
    }
    fn expr(&mut self, expr: &Expr) {
        let span = expr.span.shift(self.base);
        // A bare generator is a fold's argument, not a value a hole can move.
        if !expr.is_generator() {
            self.exprs.push(span);
        }
        if let Some(v) = crate::rules::constant(expr) {
            self.consts.insert((span.start, span.end), v);
        }
        if let ExprKind::Keep { target, .. } = &expr.kind {
            self.places.push(target.span.shift(self.base).start);
        }
    }
    fn ty(&mut self, ty: &Type) {
        self.types.push(ty.span.shift(self.base));
    }
    fn stmt(&mut self, stmt: &Stmt) {
        self.stmts.push(stmt.span.shift(self.base));
        match &stmt.kind {
            StmtKind::Assign { target, .. } => self.places.push(target.span.shift(self.base).start),
            StmtKind::Let { pat, .. } | StmtKind::For { pat, .. } => {
                self.pats.push(pat.span.shift(self.base))
            }
            StmtKind::Fn(decl) => {
                (self.fns).push((decl.name.text.clone(), stmt.span.shift(self.base)))
            }
            _ => {}
        }
    }
}

/// Whether a library lint `code` is shown in `wider`'s rewrite as often as
/// in the code it replaces: the rewrite keeps the code the lint is about (a
/// hand pop loop that becomes `sum(-(h.pop()!) for ..)`), so both findings
/// teach. Each edit's text is matched as written, dedented to its column.
pub(crate) fn refires(
    src: &str,
    wider: &cpl_diag::Diagnostic,
    code: Code,
    shown: &[cpl_diag::Diagnostic],
    declared: &[Declared],
) -> bool {
    let mine: Vec<Declared> = (declared.iter())
        .filter(|d| d.lint.code == code)
        .cloned()
        .collect();
    let Some(fix) = wider.fixes.first().filter(|_| !mine.is_empty()) else {
        return false;
    };
    let replaced = (shown.iter())
        .filter(|d| d.code == code && fix.edits.iter().any(|(at, _)| at.contains(d.span)))
        .count();
    let kept: usize = (fix.edits.iter())
        .map(|(at, to)| {
            let line = src[..at.start as usize].rfind('\n').map_or(0, |i| i + 1);
            let col = at.start as usize - line;
            let text: Vec<&str> = (to.split('\n').enumerate())
                .map(|(i, l)| {
                    let indent = l.len() - l.trim_start_matches(' ').len();
                    &l[if i == 0 { 0 } else { indent.min(col) }..]
                })
                .collect();
            let text = text.join("\n");
            let parsed = cpl_syntax::parse_file(&text);
            let items: Vec<(u32, &cpl_syntax::ParsedItem)> =
                parsed.items.iter().map(|(at, item)| (*at, item)).collect();
            let file = File {
                src: &text,
                items: &items,
                facts: None,
                proven: &Default::default(),
            };
            find(&file, &mine).len()
        })
        .sum();
    replaced > 0 && kept >= replaced
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
    // A statement's span can run past its last token (a block's newline).
    let before = |at: u32| toks.partition_point(|t| t.span.start < at);
    let mut ends: HashMap<u32, Vec<u32>> = HashMap::new();
    for e in (spans.exprs.iter()).chain(&spans.types).chain(&spans.pats) {
        ends.entry(e.start).or_default().push(e.end);
    }
    for s in &spans.stmts {
        if let Some(last) = before(s.end).checked_sub(1) {
            ends.entry(s.start).or_default().push(toks[last].span.end);
        }
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
        let to = library_lint::tokens(&d.decl.to);
        let to_labels = library_lint::labels(&to.iter().map(|t| t.kind).collect::<Vec<_>>());
        let in_to = |h: &str| {
            (to.iter().zip(&to_labels)).any(|(t, label)| {
                !label && &d.decl.to[t.span.start as usize..t.span.end as usize] == h
            })
        };
        // R11: a compound `to` runs its right side before the place's indices.
        let compound = library_lint::code(&d.decl.to)
            .is_some_and(|to| library_lint::compound(&to, &d.decl.fn_name));
        // With no fix nothing is dropped, so `to` may leave out any hole.
        let fixes = d.lint.no_fix.is_none();
        let to_kinds: Vec<(K, String)> = (to.iter())
            .map(|t| {
                (
                    t.kind,
                    d.decl.to[t.span.start as usize..t.span.end as usize].into(),
                )
            })
            .collect();
        let repeated: Vec<bool> = (d.decl.holes.iter())
            .map(|h| {
                let at = |i: &usize| d.from[*i].1 == *h && !d.labels[*i];
                let uses = (0..d.from.len()).filter(at).count();
                compound
                    || uses > 1
                    || (fixes && !in_to(h))
                    || (per_item(&d.from, h) && !per_item(&to_kinds, h))
            })
            .collect();
        let matcher = |pat, stmt, labels| Matcher {
            pat,
            stmt,
            labels,
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
                matcher(
                    &d.from[at.clone()],
                    &d.stmt_holes[at.clone()],
                    &d.labels[at],
                )
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
        let reads = d.reads.iter().map(|r| &r.0);
        let plain = vec![false; d.also.iter().chain(reads).fold(0, |n, p| n.max(p.len()))];
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
                || (fixes
                    && d.decl.receiver.is_none()
                    && (spans.fns.iter()).any(|(f, at)| *f == d.decl.fn_name && at.contains(span)))
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
                    .is_some_and(|run| proven(file, &spans.consts, &toks, &text, run, b))
            };
            if !matched
                || !d.decl.when.iter().all(bounded)
                || !(holes.iter().enumerate()).all(|(h, run)| run.is_none_or(|r| typed(h, r)))
            {
                continue;
            }
            let mut used = d.also.is_empty();
            for pat in &d.also {
                let m = Matcher {
                    repeated: vec![false; d.decl.holes.len()],
                    ..matcher(pat, &plain[..pat.len()], &plain[..pat.len()])
                };
                used |= candidates.iter().any(|c| {
                    let (mut holes, mut names) = (holes.clone(), names.clone());
                    m.go(0, c.0, c.1, &mut holes, &mut names)
                });
            }
            if !used {
                continue;
            }
            // A method's lint another type's method also declares holds
            // only on its own type's values (a rewrite's text, untyped,
            // keeps it).
            let shared = |ty: &String| {
                (declared.iter()).any(|o| {
                    o.lint.code == d.lint.code && o.decl.receiver.as_ref().is_some_and(|r| r != ty)
                })
            };
            let receiver =
                (d.decl.receiver.as_ref()).filter(|ty| file.facts.is_some() && shared(ty));
            let receiver = receiver.and_then(|ty| {
                let at = d.decl.holes.iter().position(|h| h == "self");
                let (a, b) = at.and_then(|i| holes[i])?;
                Some((Span::new(toks[a].span.start, toks[b - 1].span.end), ty))
            });
            let place = |at: Span| file.facts.and_then(|f| f.place_type(at));
            if receiver.is_some_and(|(at, ty)| !place(at).is_some_and(|t| t.is_one_of(&[ty]))) {
                continue;
            }
            let first = before(span.start);
            let commented = (first..stop - 1).any(|i| {
                file.src[toks[i].span.end as usize..toks[i + 1].span.start as usize].contains("//")
            });
            // `to`'s `let p = ..` can't use code that reads `p`: that `p`
            // is not declared yet.
            let named = |n: &str| {
                let hole = d.decl.holes.iter().position(|h| h == n);
                match hole.and_then(|h| holes[h]) {
                    Some((a, b)) => Some(text[a..b].concat()),
                    None => names.iter().find(|u| u.0 == n).map(|u| u.1.clone()),
                }
            };
            let reads_own = d.inits.iter().any(|(n, hs)| {
                named(n).is_some_and(|user| {
                    (hs.iter()
                        .filter_map(|h| holes[d.decl.holes.iter().position(|x| x == h)?]))
                    .any(|(a, b)| (a..b).any(|j| kinds[j] == K::Ident && text[j] == user))
                })
            });
            if reads_own {
                continue;
            }
            let got_of = |holes: &[Option<(usize, usize)>]| -> HashMap<&str, &str> {
                (d.decl.holes.iter().zip(holes))
                    .filter_map(|(h, run)| {
                        let (a, b) = (*run)?;
                        Some((
                            h.as_str(),
                            &file.src[toks[a].span.start as usize..toks[b - 1].span.end as usize],
                        ))
                    })
                    .collect()
            };
            let got = got_of(&holes);
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
            let subject = (to.iter().zip(&to_labels))
                .filter(|(_, label)| !**label)
                .map(|(t, _)| &d.decl.to[t.span.start as usize..t.span.end as usize])
                .find_map(|w| holes[d.decl.holes.iter().position(|h| h == w)?])
                .or_else(|| holes[d.receiver?]);
            let from = if d.module.is_empty() {
                "this file".to_string()
            } else {
                format!("`{}`", d.module)
            };
            // With no fix the statements a hole matched stay, so the finding
            // ends before them and the findings inside them stand.
            let kept = (d.from.iter().zip(&d.stmt_holes))
                .filter(|(_, s)| **s)
                .filter_map(|((_, h), _)| holes[d.decl.holes.iter().position(|x| x == h)?])
                .map(|(a, _)| a)
                .min();
            let shown = match kept {
                Some(a) if d.lint.no_fix.is_some() && a > 0 => {
                    Span::new(span.start, toks[a - 1].span.end)
                }
                _ => span,
            };
            let mut diag = cpl_diag::msgs::lint_rules::LIBRARY_LINT_FIX
                .warning_as(d.lint.code, shown, &[&fix])
                .with_help(format!(
                    "{from} declares this lint with `@lint` on `{}`",
                    d.decl.fn_name
                ));
            if let Some((a, b)) = subject {
                let at = Span::new(toks[a].span.start, toks[b - 1].span.end);
                diag = diag.with_label(at, format!("what `{}` takes", d.decl.fn_name));
            }
            let mut fixed = diag.clone();
            let fix_title = if d.decl.drops {
                format!("drop `{}`", d.decl.fn_name)
            } else {
                format!("use `{}`", d.decl.suggests)
            };
            fixed.fix(fix_title, span, fix);
            // `reads:` rewrites every later use of the name; one that is no
            // such read (a write, `p.len`) leaves the code unfixed.
            let reads = d.reads.as_ref().map(|(pat, write)| {
                let m = Matcher {
                    repeated: vec![false; d.decl.holes.len()],
                    ..matcher(pat, &plain[..pat.len()], &plain[..pat.len()])
                };
                let (_, user) = (pat.iter()).find_map(|(_, t)| names.iter().find(|n| n.0 == *t))?;
                let last = site[site.len() - 1];
                let end = block_end(&spans.blocks, last);
                let mut edits: Vec<(Span, String)> = Vec::new();
                for j in (0..toks.len()).filter(|&j| {
                    toks[j].span.start >= last.end
                        && toks[j].span.end <= end
                        && kinds[j] == K::Ident
                        && text[j] == user
                        && (j == 0 || kinds[j - 1] != K::Dot)
                }) {
                    if edits.last().is_some_and(|e| e.0.end > toks[j].span.start) {
                        return None;
                    }
                    let (mut h, mut n) = (holes.clone(), names.clone());
                    let c = candidates.iter().find(|c| {
                        (h, n) = (holes.clone(), names.clone());
                        c.0 == j && m.go(0, c.0, c.1, &mut h, &mut n)
                    })?;
                    edits.push((
                        c.2,
                        fill(write, &got_of(&h), &n, &d.spell, "", &HashMap::new()),
                    ));
                }
                Some(edits)
            });
            match reads {
                Some(None) => fixed = diag.clone(),
                Some(Some(edits)) => fixed.fixes[0].edits.extend(edits),
                None => {}
            }
            if !d.rebinds.is_empty()
                && !rebinding_holds(
                    file.src,
                    &toks,
                    &spans.blocks,
                    &site,
                    &d.rebinds,
                    &names,
                    &fixed,
                )
            {
                continue;
            }
            if !commented && d.lint.no_fix.is_none() && !d.decl.advice {
                diag = fixed;
            }
            taken.push(span);
            out.push(Finding { lint: d.lint, diag });
        }
    }
    out
}

/// Whether a fix that declares again a name the code declared (`var m:
/// Map<K, Int> = Map()` -> `let m = xs.tally()`) keeps the program: the
/// rest of the block uses the name only through a member or an index, never
/// whole (`print(m)` prints a `Counter` in another order), and the fixed
/// file checks as the code did (a `.first` a `Counter` lacks).
fn rebinding_holds(
    src: &str,
    toks: &[Token],
    blocks: &[Vec<Span>],
    site: &[Span],
    rebinds: &[String],
    names: &[(String, String)],
    diag: &cpl_diag::Diagnostic,
) -> bool {
    let last = site[site.len() - 1];
    let end = block_end(blocks, last);
    let text = |t: &Token| &src[t.span.start as usize..t.span.end as usize];
    let rest: Vec<&Token> = (toks.iter())
        .filter(|t| t.span.start >= last.end && t.span.end <= end)
        .collect();
    let whole = (rebinds.iter())
        .filter_map(|n| names.iter().find(|(p, _)| p == n).map(|(_, u)| u.as_str()))
        .any(|user| {
            rest.iter().enumerate().any(|(i, t)| {
                t.kind == K::Ident
                    && text(t) == user
                    && (i == 0 || rest[i - 1].kind != K::Dot)
                    && !rest
                        .get(i + 1)
                        .is_some_and(|n| matches!(n.kind, K::Dot | K::LBracket))
            })
        });
    !whole && crate::recheck::fix_keeps_verdict(src, diag)
}

/// Where the block holding `stmt` ends.
fn block_end(blocks: &[Vec<Span>], stmt: Span) -> u32 {
    (blocks.iter())
        .filter(|b| b.contains(&stmt))
        .filter_map(|b| b.last().map(|s| s.end))
        .max()
        .unwrap_or(stmt.end)
}

struct Matcher<'a> {
    pat: &'a [(K, String)],
    /// Per token of `pat`: a hole that matches a run of statements.
    stmt: &'a [bool],
    /// Per token of `pat`, whether it is a named argument's label.
    labels: &'a [bool],
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
        if is_name(*kind, text) && !self.labels[p] {
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
                } else if p > 0 && self.pat[p - 1].0 == K::Dot {
                    // After `.`, a hole is a field or method name.
                    (self.kinds[j] == K::Ident)
                        .then_some(j + 1)
                        .into_iter()
                        .collect()
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

    /// The ends of the expressions, types and binding patterns starting at
    /// token `j` and ending by `stop`, a bracketed one with its brackets.
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
/// whose ends lie in it (`k: 1..=10` for `k >= 1`), or a constant
/// expression (`10**18`) whose value does.
fn proven(
    file: &File,
    consts: &HashMap<(u32, u32), i128>,
    toks: &[Token],
    text: &[&str],
    run: (usize, usize),
    b: &Bound,
) -> bool {
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
            if let Some(&v) = consts.get(&(span.start, span.end)) {
                return b.lo.is_none_or(|want| v >= want) && b.hi.is_none_or(|want| v <= want);
            }
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
/// Whether `toks` evaluate `h` once per item of a comprehension: anywhere in
/// its brackets but the iterable of its first `for` (`c` and `m` in
/// `[[c for _ in 0..m] for _ in 0..n]`, not `n`).
fn per_item(toks: &[(K, String)], h: &str) -> bool {
    let mut open = Vec::new();
    let mut close = vec![toks.len(); toks.len()];
    for (i, (k, _)) in toks.iter().enumerate() {
        match k {
            K::LParen | K::LBracket | K::LBrace => open.push(i),
            K::RParen | K::RBracket | K::RBrace => {
                if let Some(o) = open.pop() {
                    close[o] = i;
                }
            }
            _ => {}
        }
    }
    let clauses = |o: usize| {
        let (mut depth, mut seen) = (0, Vec::new());
        for (i, (k, _)) in toks.iter().enumerate().take(close[o]).skip(o + 1) {
            match k {
                K::LParen | K::LBracket | K::LBrace => depth += 1,
                K::RParen | K::RBracket | K::RBrace => depth -= 1,
                K::For | K::In | K::If if depth == 0 => seen.push((i, *k)),
                _ => {}
            }
        }
        seen
    };
    (toks.iter().enumerate())
        .filter(|(_, (k, t))| is_name(*k, t) && t == h)
        .any(|(i, _)| {
            (0..i)
                .filter(|&o| matches!(toks[o].0, K::LParen | K::LBracket | K::LBrace))
                .filter(|&o| i < close[o])
                .any(|o| {
                    let seen = clauses(o);
                    let Some(f) = seen.iter().position(|c| c.1 == K::For) else {
                        return false;
                    };
                    let Some(at) = seen[f..].iter().position(|c| c.1 == K::In) else {
                        return true;
                    };
                    let from = seen[f + at].0;
                    let to = seen[f + at + 1..].first().map_or(close[o], |c| c.0);
                    !(from < i && i < to)
                })
        })
}

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
    let labels = library_lint::labels(&toks.iter().map(|t| t.kind).collect::<Vec<_>>());
    for (i, t) in toks.iter().enumerate() {
        if labels[i] {
            continue;
        }
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
            let labelled = before == Some(K::Colon) && i >= 2 && labels[i - 2];
            let delimited = ((labelled
                || matches!(before, Some(K::LParen | K::LBracket | K::Comma) | None))
                && matches!(after, Some(K::RParen | K::RBracket | K::Comma) | None))
                || (before == Some(K::Eq) && after.is_none())
                || (before == Some(K::LBracket)
                    && after == Some(K::For)
                    && !clauses(m.trim_matches(['(', ')'])));
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
/// Whether `src` has a `for`, `if` or `=>` outside brackets, which would
/// run on into a comprehension's `for` (`[e for i in r]`).
fn clauses(src: &str) -> bool {
    let mut depth = 0;
    library_lint::tokens(src).iter().any(|t| {
        match t.kind {
            K::LParen | K::LBracket => depth += 1,
            K::RParen | K::RBracket => depth -= 1,
            _ => {}
        }
        depth == 0 && matches!(t.kind, K::For | K::If | K::FatArrow)
    })
}

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
    fn a_hole_is_per_item_inside_a_comprehension_but_its_first_iterable() {
        let toks = |src: &str| {
            (cpl_syntax::library_lint::tokens(src).iter())
                .map(|t| {
                    (
                        t.kind,
                        src[t.span.start as usize..t.span.end as usize].to_string(),
                    )
                })
                .collect::<Vec<_>>()
        };
        let grid = toks("[[c for _ in 0..m] for _ in 0..n if p]");
        let each: Vec<bool> = ["c", "m", "n", "p"]
            .iter()
            .map(|h| super::per_item(&grid, h))
            .collect();
        assert_eq!(each, [true, true, false, true]);
        assert!(!super::per_item(&toks("[n, m].filled(c)"), "c"));
    }

    #[test]
    fn brackets_balance_only_when_each_closes_one_opened_before_it() {
        assert!(super::balanced("f(x) + a[i]"));
        assert!(!super::balanced("a) + (b"));
        assert!(!super::balanced("(a"));
    }
}
