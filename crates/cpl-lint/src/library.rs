//! Lints a library declares with `@lint(name, from: "...", to: "...")` on
//! one of its fns (R06.4.9). A program that uses the library, and the
//! library itself outside the fn, gets a finding where it writes `from`, with
//! `to` as its fix. The fn's parameter names are the holes.

use crate::walk::Visitor;
use crate::{File, Finding, Group, Level, Lint};
use cpl_diag::{Code, Span};
use cpl_facade::today::Analysis;
use cpl_syntax::ast::{Cond, Expr, ExprKind, Stmt, StmtKind, Type};
use cpl_syntax::library_lint::{self, is_name, Bound, LibraryLint};
use cpl_syntax::{Token, TokenKind as K};
use std::collections::HashMap;
use std::sync::Mutex;

/// One `@lint` declaration a file can see.
pub(crate) struct Declared {
    pub lint: &'static Lint,
    pub(crate) decl: LibraryLint,
    /// `std.seq`; empty for the file itself.
    pub(crate) module: String,
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
    /// Whether `reads:`'s name is a loop variable `from` binds: its reads
    /// are in the statement holes, rewritten in what those matched.
    loop_read: bool,
    /// Whether `to` drops `reads:`'s name: every use must then be a read.
    drops_read: bool,
    /// `each:`: the name `to` writes for an element, and its hole.
    each: Option<(String, usize)>,
    /// Per name `to` declares, the holes its initializer uses.
    inits: Vec<(String, Vec<String>)>,
    /// `to` as one run of code, and whether it is one statement.
    to_toks: Vec<(K, String)>,
    to_single: bool,
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
            declare_all(
                &mut out,
                item.offset,
                &item.syntax.parsed.stmts,
                module,
                spell,
            );
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
            // A method imported under its key (`max of std.seq`) is spelled
            // as written; only a plain name is a spelling the file can write.
            let mut imported = (analysis.imports().iter())
                .filter(|(_, imp)| imp.module == i && imp.name == w)
                .map(|(local, _)| local)
                .peekable();
            if imported.peek().is_some() {
                let local = imported.find(|l| l.chars().all(|c| c.is_alphanumeric() || c == '_'));
                return local.filter(|l| *l != w).cloned();
            }
            let (alias, _) = analysis.aliases().iter().find(|(_, &at)| at == i)?;
            Some(format!("{alias}.{w}"))
        };
        add(&m.analysis, &m.name, &spell);
    }
    // Std's from its outline: a program that hand-writes what a std fn does
    // seldom loads its module, and std's names need no `use`.
    for (name, outline) in cpl_facade::Std.outlines() {
        let module = format!("std.{}", name.replace('/', "."));
        if !outline.contains("@lint(") || analysis.modules().iter().any(|m| m.name == module) {
            continue;
        }
        for (offset, item) in &cpl_syntax::parse_file(outline).items {
            declare_all(&mut out, *offset, &item.stmts, &module, &|_| None);
        }
    }
    out
}

/// What `collect` keeps of the `@lint`s on an item's statements.
fn declare_all(
    out: &mut Vec<Declared>,
    offset: u32,
    stmts: &[Stmt],
    module: &str,
    spell: &dyn Fn(&str) -> Option<String>,
) {
    for stmt in stmts {
        for decl in library_lint::declared(stmt, offset).0 {
            out.extend(declare(decl, module, spell));
        }
    }
}

/// What `collect` keeps of `decl`: none when its `from` is no run of code.
fn declare(
    decl: LibraryLint,
    module: &str,
    spell: &dyn Fn(&str) -> Option<String>,
) -> Option<Declared> {
    let (from, ends) = library_lint::run(&decl.from)?;
    let names = library_lint::statement_names(&decl.from);
    let (mut stmt_holes, mut kinds) = (Vec::new(), Vec::new());
    for (i, t) in from.iter().enumerate() {
        stmt_holes.push(names.contains(&i) && decl.holes.contains(&t.1));
        kinds.push(t.0);
    }
    let mut spelled = Vec::new();
    let to_tokens = library_lint::tokens(&decl.to);
    for (i, t) in to_tokens.iter().enumerate() {
        let w = &decl.to[t.span.start as usize..t.span.end as usize];
        // `x max= e` writes the language's operator, not the module's `max`,
        // and `l lcm= x` takes a name, never a path.
        let op =
            (to_tokens.get(i + 1)).is_some_and(|n| n.kind == K::Eq && n.span.start == t.span.end);
        let writable = |s: &String| {
            !op || (!matches!(w, "min" | "max")
                && s.chars().all(|c| c.is_alphanumeric() || c == '_'))
        };
        if let Some(s) = spell(w).filter(writable) {
            spelled.push((w.to_string(), s));
        }
    }
    let labels = cpl_next_syntax::arg_labels(&kinds);
    let mut receiver = None;
    for i in 1..from.len() {
        if receiver.is_none() && !labels[i - 1] && from[i].0 == K::Dot {
            receiver = decl.holes.iter().position(|h| *h == from[i - 1].1);
        }
    }
    fn declared_in(toks: &[(K, String)], typed: bool) -> Vec<&String> {
        let mut names = Vec::new();
        for w in toks.windows(3) {
            if matches!(w[0].0, K::Let | K::Var)
                && w[1].0 == K::Ident
                && (w[2].0 == K::Colon) == typed
            {
                names.push(&w[1].1);
            }
        }
        names
    }
    let (to_toks, to_ends) = library_lint::run(&decl.to).unwrap_or_default();
    let to_names = declared_in(&to_toks, false);
    let mut rebinds = Vec::new();
    for n in declared_in(&from, true) {
        if to_names.contains(&n) {
            rebinds.push(n.clone());
        }
    }
    let reads = (decl.reads.split_once("->"))
        .and_then(|(r, w)| Some((library_lint::code(r.trim())?, w.trim().to_string())));
    let each = (decl.each.split_once(" in ")).and_then(|(x, xs)| {
        Some((
            x.trim().to_string(),
            decl.holes.iter().position(|h| h == xs.trim())?,
        ))
    });
    let read_name = (reads.as_ref())
        .and_then(|(pat, _)| pat.iter().find(|t| bound_names(&from).contains(&t.1)))
        .map(|t| t.1.clone());
    let loop_read = (read_name.as_ref())
        .is_some_and(|n| from.windows(2).any(|w| w[0].0 == K::For && w[1].1 == *n));
    let drops_read = (read_name.as_ref()).is_some_and(|n| !to_toks.iter().any(|t| t.1 == *n));
    let mut inits = Vec::new();
    let mut start = 0;
    for &end in &to_ends {
        let stmt = &to_toks[std::mem::replace(&mut start, end)..end];
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
        spell: spelled,
        lint: intern(&decl.name),
        bound: bound_names(&from),
        stmt_holes,
        labels,
        receiver,
        rebinds,
        reads,
        loop_read,
        drops_read,
        each,
        inits,
        to_single: to_ends.len() == 1,
        to_toks,
        decl,
        module: module.to_string(),
        from,
        ends,
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
    /// Each branch of an `if` with an `elif` or `else`: where its test
    /// starts and its body ends. A branch matches an `if` from alone.
    branches: Vec<(u32, u32)>,
}

impl Visitor for Spans {
    fn stmts(&mut self, stmts: &[(u32, &Stmt)]) {
        let base = self.base;
        self.blocks
            .push(stmts.iter().map(|(b, s)| s.span.shift(base + b)).collect());
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
        if let ExprKind::If {
            branches,
            otherwise,
        } = &expr.kind
        {
            for (cond, body) in branches
                .iter()
                .filter(|_| branches.len() > 1 || otherwise.is_some())
            {
                let test = match cond {
                    Cond::Expr(e) | Cond::Const(e) | Cond::Guard { test: e, .. } => e.span,
                    Cond::Let { pat, .. } => pat.span,
                };
                if let Some(last) = body.stmts.last() {
                    (self.branches)
                        .push((test.shift(self.base).start, last.span.shift(self.base).end));
                }
            }
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
    let mine = declared.iter().any(|d| d.lint.code == code);
    let Some(fix) = wider.fixes.first().filter(|_| mine) else {
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
            find(&file, declared, Some(code)).len()
        })
        .sum();
    replaced > 0 && kept >= replaced
}

/// The findings of `declared`'s lints in `file`, of `only`'s alone when given.
pub(crate) fn find(file: &File, declared: &[Declared], only: Option<Code>) -> Vec<Finding> {
    if declared.is_empty() {
        return Vec::new();
    }
    let toks = library_lint::tokens(file.src);
    let (mut text, mut kinds) = (Vec::new(), Vec::new());
    let (mut by_start, mut by_end) = (HashMap::new(), HashMap::new());
    for (i, t) in toks.iter().enumerate() {
        text.push(&file.src[t.span.start as usize..t.span.end as usize]);
        kinds.push(t.kind);
        by_start.insert(t.span.start, i);
        by_end.insert(t.span.end, i + 1);
    }
    let mut spans = Spans::default();
    file.walk(&mut spans, |s, base| s.base = base);
    // A statement's span can run past its last token (a block's newline).
    // How many tokens start before each offset, looked up once per site and lint.
    let mut starting_before = Vec::with_capacity(file.src.len() + 1);
    for at in 0..=file.src.len() as u32 {
        let seen = starting_before.last().copied().unwrap_or(0);
        let more = toks[seen..]
            .iter()
            .take_while(|t| t.span.start < at)
            .count();
        starting_before.push(seen + more);
    }
    let before = |at: u32| starting_before[(at as usize).min(file.src.len())];
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
    // An `if` branch from its keyword: an `elif` matches `from`'s `if`.
    let branches: Vec<Span> = (spans.branches.iter())
        .filter_map(|&(test, end)| {
            let kw = (0..before(test))
                .rev()
                .find(|&i| matches!(kinds.get(i), Some(K::If | K::Elif)))?;
            Some(Span::new(toks[kw].span.start, end))
        })
        .collect();
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
        if only.is_some_and(|c| d.lint.code != c) {
            continue;
        }
        let to = library_lint::tokens(&d.decl.to);
        let mut to_kinds = Vec::new();
        for t in &to {
            to_kinds.push(t.kind);
        }
        let to_labels = cpl_next_syntax::arg_labels(&to_kinds);
        let mut to_words = Vec::new();
        for (t, &label) in to.iter().zip(&to_labels) {
            if !label {
                to_words.push(&d.decl.to[t.span.start as usize..t.span.end as usize]);
            }
        }
        let in_to = |h: &str| to_words.contains(&h);
        // R11: a compound `to` runs its right side before the place's indices.
        let compound = d.to_single && library_lint::compound(&d.to_toks, &d.decl.fn_name);
        // With no fix nothing is dropped, so `to` may leave out any hole.
        let fixes = d.lint.no_fix.is_none();
        let repeated: Vec<bool> = (d.decl.holes.iter())
            .map(|h| {
                let mut uses = 0;
                for (i, t) in d.from.iter().enumerate() {
                    if t.1 == *h && !d.labels[i] {
                        uses += 1;
                    }
                }
                compound
                    || uses > 1
                    || (fixes && !in_to(h))
                    || (per_item(&d.from, h) && !per_item(&d.to_toks, h))
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
            repeated: &repeated,
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
                    .chain(branches.iter().copied())
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
        let also = d.also.iter().chain(reads).fold(0, |n, p| n.max(p.len()));
        let plain = vec![false; also.max(d.decl.holes.len())];
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
            // A `from` ending in a name (`t.order`) is not the callee of a
            // call (`t.order(v)`), which names another thing.
            let callee = d.from.last().is_some_and(|(k, _)| *k == K::Ident)
                && toks.get(stop).is_some_and(|t| t.kind == K::LParen);
            if callee
                || (d.module.is_empty() && d.decl.item.contains(span))
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
                    repeated: &plain,
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
            // A method's lint another type's method also declares does not
            // hold on that type's values, which its own lint covers (any
            // other value, typed or not, keeps it).
            let others: Vec<&str> = (declared.iter())
                .filter(|o| o.lint.code == d.lint.code && o.decl.receiver != d.decl.receiver)
                .filter_map(|o| o.decl.receiver.as_deref())
                .collect();
            let at = d
                .decl
                .holes
                .iter()
                .position(|h| h == "self")
                .and_then(|i| holes[i]);
            let place = |(a, b): (usize, usize)| {
                let at = Span::new(toks[a].span.start, toks[b - 1].span.end);
                file.facts.and_then(|f| f.place_type(at))
            };
            if d.decl.receiver.is_some() && at.and_then(place).is_some_and(|t| t.is_one_of(&others))
            {
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
            // Lines `from` writes differently never match one line twice
            // (`g[u].push(v)` then `g[v].push(u)` is no self-loop's pair).
            let code: Vec<&str> = (file.src[span.start as usize..span.end as usize].lines())
                .map(str::trim)
                .collect();
            let pat: Vec<&str> = d.decl.from.lines().map(str::trim).collect();
            let twice = |l: &[&str]| (1..l.len()).any(|i| l[..i].contains(&l[i]));
            if code.len() == pat.len() && twice(&code) && !twice(&pat) {
                continue;
            }
            let got_of = |holes: &[Option<(usize, usize)>]| {
                let mut got = Vec::new();
                for (h, run) in d.decl.holes.iter().zip(holes) {
                    if let Some((a, b)) = *run {
                        let at = toks[a].span.start as usize..toks[b - 1].span.end as usize;
                        got.push((h.as_str(), &file.src[at]));
                    }
                }
                got
            };
            // `reads:` rewrites every use of its name in the name's scope:
            // the rest of the block, or a loop variable's statement holes. A
            // use that is no such read (a write, `p.len`) leaves the code
            // unfixed; where `to` drops the name, it is no such code.
            // `each:` asks its hole to match a name, and spells its own as
            // that name's singular (`vals` -> `val`), one the file does not
            // use; with none such the finding has no fix.
            let plural = d.each.as_ref().map(|(_, h)| match holes[*h] {
                Some((a, b)) if b == a + 1 && kinds[a] == K::Ident => Some(text[a]),
                _ => None,
            });
            if plural == Some(None) {
                continue;
            }
            let named_each = d.each.as_ref().is_none_or(|(x, _)| {
                let plural = plural.flatten().unwrap_or_default();
                let used =
                    |o: &str| (text.iter().zip(&kinds)).any(|(t, k)| *k == K::Ident && *t == o);
                let one = (plural.strip_suffix('s')).filter(|o| o.len() > 1 && !used(o));
                one.map(|o| names.push((x.clone(), o.to_string())))
                    .is_some()
            });
            let hole_runs: Vec<(usize, usize)> = (d.from.iter().zip(&d.stmt_holes))
                .filter(|(_, s)| **s)
                .filter_map(|((_, h), _)| holes[d.decl.holes.iter().position(|x| x == h)?])
                .collect();
            let read_edits = d.reads.as_ref().map(|(pat, write)| {
                let write_toks = library_lint::tokens(write);
                let mut write_kinds = Vec::new();
                for t in &write_toks {
                    write_kinds.push(t.kind);
                }
                let write_labels = cpl_next_syntax::arg_labels(&write_kinds);
                let m = Matcher {
                    repeated: &plain[..d.decl.holes.len()],
                    ..matcher(pat, &plain[..pat.len()], &plain[..pat.len()])
                };
                let (_, user) = (pat.iter()).find_map(|(_, t)| names.iter().find(|n| n.0 == *t))?;
                let last = site[site.len() - 1];
                let end = block_end(&spans.blocks, last);
                let in_scope = |j: usize| match d.loop_read {
                    true => hole_runs.iter().any(|&(a, b)| (a..b).contains(&j)),
                    false => toks[j].span.start >= last.end && toks[j].span.end <= end,
                };
                let mut edits: Vec<(Span, String)> = Vec::new();
                for j in (0..toks.len()).filter(|&j| {
                    in_scope(j)
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
                        c.0 <= j && j < c.1 && m.go(0, c.0, c.1, &mut h, &mut n)
                    })?;
                    edits.push((
                        c.2,
                        fill(
                            write,
                            (&write_toks, &write_labels),
                            &got_of(&h),
                            &n,
                            &d.spell,
                            "",
                            &[],
                        ),
                    ));
                }
                // What the read's holes matched stays what it was: a name of
                // it is used in the scope only in the reads (not assigned,
                // declared again, or read whole).
                let kept: Vec<&str> = (pat.iter())
                    .filter_map(|(_, t)| holes[d.decl.holes.iter().position(|h| h == t)?])
                    .filter(|(a, b)| b - a == 1)
                    .map(|(a, _)| text[a])
                    .collect();
                let in_read = |j: usize| edits.iter().any(|e| e.0.contains(toks[j].span));
                if (0..toks.len()).any(|j| in_scope(j) && kept.contains(&text[j]) && !in_read(j)) {
                    return None;
                }
                (!d.drops_read || !edits.is_empty()).then_some(edits)
            });
            if d.drops_read && matches!(read_edits, Some(None)) {
                continue;
            }
            // A loop variable's reads are rewritten in what its statement
            // holes matched.
            let subbed: Vec<(&str, String)> = match (d.loop_read, &read_edits) {
                (true, Some(Some(edits))) => (d.from.iter().zip(&d.stmt_holes))
                    .filter(|(_, s)| **s)
                    .filter_map(|((_, h), _)| {
                        let (a, b) = holes[d.decl.holes.iter().position(|x| x == h)?]?;
                        let (start, stop) = (toks[a].span.start, toks[b - 1].span.end);
                        let mut out = String::new();
                        let mut at = start;
                        for (span, w) in edits
                            .iter()
                            .filter(|e| e.0.start >= start && e.0.end <= stop)
                        {
                            out += &file.src[at as usize..span.start as usize];
                            out += w;
                            at = span.end;
                        }
                        out += &file.src[at as usize..stop as usize];
                        Some((h.as_str(), out))
                    })
                    .collect(),
                _ => Vec::new(),
            };
            // A rewritten hole comes first, so it is the one `fill` finds.
            let mut got: Vec<(&str, &str)> = subbed.iter().map(|(h, s)| (*h, s.as_str())).collect();
            got.extend(got_of(&holes));
            let column = |at: u32| {
                let line = file.src[..at as usize].rfind('\n').map_or(0, |i| i + 1);
                (line, at as usize - line)
            };
            // With no fix the statements a hole matched stay, so the finding
            // ends before them and the findings inside them stand.
            let (mut stmts, mut kept) = (Vec::new(), usize::MAX);
            for ((_, h), &stmt) in d.from.iter().zip(&d.stmt_holes) {
                let at = d.decl.holes.iter().position(|x| x == h);
                if let Some((a, _)) = at.and_then(|at| holes[at]).filter(|_| stmt) {
                    stmts.push((h.as_str(), column(toks[a].span.start).1));
                    kept = kept.min(a);
                }
            }
            let line = column(span.start).0;
            let indent: String = (file.src[line..].chars())
                .take_while(|c| *c == ' ' || *c == '\t')
                .collect();
            let fix = fill(
                &d.decl.to,
                (&to, &to_labels),
                &got,
                &names,
                &d.spell,
                &indent,
                &stmts,
            );
            let fix = match kinds.get(before(span.start)) {
                Some(K::Elif) => format!("el{fix}"),
                _ => fix,
            };
            let subject = (to_words.iter())
                .find_map(|w| holes[d.decl.holes.iter().position(|h| h == w)?])
                .or_else(|| holes[d.receiver?]);
            let from = if d.module.is_empty() {
                "this file".to_string()
            } else {
                format!("`{}`", d.module)
            };
            let shown = match kept {
                a if d.lint.no_fix.is_some() && a > 0 && a < usize::MAX => {
                    Span::new(span.start, toks[a - 1].span.end)
                }
                _ => span,
            };
            let mut diag = cpl_diag::msgs::lint_rules::LIBRARY_LINT_FIX
                .warning_as(d.lint.code, shown, &[&first_line(&fix)])
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
            match read_edits {
                Some(None) => fixed = diag.clone(),
                Some(Some(edits)) if !d.loop_read => fixed.fixes[0].edits.extend(edits),
                _ => {}
            }
            if !named_each {
                fixed = diag.clone();
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
    repeated: &'a [bool],
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
        let elif = p == 0 && *kind == K::If && self.kinds[j] == K::Elif;
        (elif || (self.kinds[j] == *kind && self.text[j] == text))
            && self.go(p + 1, j + 1, stop, holes, names)
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
                Some(cpl_facade::today::Ty::Range { lo, hi, .. }) => (*lo, *hi),
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
    (toks, labels): (&[cpl_syntax::Token], &[bool]),
    got: &[(&str, &str)],
    names: &[(String, String)],
    spell: &[(String, String)],
    indent: &str,
    stmts: &[(&str, usize)],
) -> String {
    let mut out = String::new();
    let mut last = 0;
    let lines = |s: &str| s.replace('\n', &format!("\n{indent}"));
    for (i, t) in toks.iter().enumerate() {
        if labels[i] {
            continue;
        }
        let word = &to[t.span.start as usize..t.span.end as usize];
        let got = got.iter().find(|(h, _)| *h == word).map(|(_, m)| *m);
        let was = stmts.iter().find(|(h, _)| *h == word).map(|(_, w)| *w);
        let replacement = if let (Some(m), Some(was)) = (got, was) {
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
        } else if let Some(m) = got.filter(|_| is_name(t.kind, word)) {
            let before = i.checked_sub(1).map(|i| toks[i].kind);
            let after = toks.get(i + 1).map(|t| t.kind);
            let labelled = before == Some(K::Colon) && i >= 2 && labels[i - 2];
            let delimited = ((labelled
                || matches!(before, Some(K::LParen | K::LBracket | K::Comma) | None))
                && matches!(after, Some(K::RParen | K::RBracket | K::Comma) | None))
                || (before == Some(K::Eq) && after.is_none())
                || (before == Some(K::LBracket)
                    && after == Some(K::For)
                    && !clauses(m.trim_matches(['(', ')'])))
                || (matches!(before, Some(K::If | K::Elif | K::While)) && after == Some(K::Colon));
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

/// A fix's first line, `...` standing for the rest: a message is one line,
/// as every tool shows it.
fn first_line(fix: &str) -> String {
    match fix.split_once('\n') {
        Some((first, _)) => format!("{} ...", first.trim_end()),
        None => fix.to_string(),
    }
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
