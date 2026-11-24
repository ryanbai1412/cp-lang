//! Reading the syntax tree: which statements and expressions read, and what.

use super::*;

/// Whether `a` and `b` are the same arithmetic on the same names, whatever
/// their spans.
pub(super) fn same(a: &Expr, b: &Expr) -> bool {
    match (&a.kind, &b.kind) {
        (ExprKind::Paren(a), _) => same(a, b),
        (_, ExprKind::Paren(b)) => same(a, b),
        (ExprKind::Int(x), ExprKind::Int(y)) => x == y,
        (ExprKind::Name(x), ExprKind::Name(y)) => x == y,
        (
            ExprKind::Binary { op, lhs, rhs, .. },
            ExprKind::Binary {
                op: op2,
                lhs: lhs2,
                rhs: rhs2,
                ..
            },
        ) => op == op2 && same(lhs, lhs2) && same(rhs, rhs2),
        _ => false,
    }
}

/// The names of the types `ty` is made of: `V` and `Int` in `[V]Int`.
pub(super) fn type_names(ty: &Type, out: &mut HashSet<String>) {
    match &ty.kind {
        TypeKind::Named { name, args } => {
            out.insert(name.text.clone());
            args.iter().for_each(|arg| type_names(arg, out));
        }
        TypeKind::Array { index, elem } => {
            index.iter().for_each(|ty| type_names(ty, out));
            type_names(elem, out);
        }
        TypeKind::OwnDomain { elem: ty, .. } | TypeKind::Optional(ty) | TypeKind::Mut(ty) => {
            type_names(ty, out)
        }
        TypeKind::Tuple(tys) => tys.iter().for_each(|ty| type_names(ty, out)),
        TypeKind::Record(fields) => fields.iter().for_each(|(_, ty)| type_names(ty, out)),
        _ => {}
    }
}

pub(super) fn bind_names(pat: &Pattern, out: &mut HashSet<String>) {
    match &pat.kind {
        PatternKind::Name(name) => {
            out.insert(name.clone());
        }
        PatternKind::Domain(name) => {
            out.insert(name.text.clone());
        }
        PatternKind::Tuple(pats)
        | PatternKind::Or(pats)
        | PatternKind::Variant { args: pats, .. } => {
            pats.iter().for_each(|pat| bind_names(pat, out))
        }
        _ => {}
    }
}

/// The jumps that leave or skip the rest of one pass of a loop: a
/// `break` or `continue` inside a nested loop only ends that loop, and a
/// `return` inside a nested fn or closure only ends that call.
#[derive(Default)]
pub(super) struct Jumps {
    loops: Vec<Span>,
    calls: Vec<Span>,
    breaks: Vec<(Span, bool)>,
    continues: Vec<(Span, bool)>,
    returns: Vec<Span>,
}

impl Jumps {
    pub(super) fn of(stmt: &Stmt) -> Jumps {
        let mut jumps = Jumps::default();
        cpl_syntax::visit::walk_stmt(stmt, &mut jumps);
        jumps
    }

    fn within(spans: &[Span], at: Span) -> bool {
        spans.iter().any(|s| s.start <= at.start && at.end <= s.end)
    }

    fn escapes(&self, jumps: &[(Span, bool)]) -> bool {
        jumps
            .iter()
            .any(|&(at, labeled)| labeled || !Self::within(&self.loops, at))
    }

    /// A `break` or `return` that ends the loop this pass is in.
    pub(super) fn leaves(&self) -> bool {
        self.escapes(&self.breaks)
            || self
                .returns
                .iter()
                .any(|&at| !Self::within(&self.calls, at))
    }

    pub(super) fn skips(&self) -> bool {
        self.escapes(&self.continues)
    }
}

impl cpl_syntax::visit::Visitor for Jumps {
    fn stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::For { .. } | StmtKind::While { .. } => self.loops.push(stmt.span),
            StmtKind::Fn(_) => self.calls.push(stmt.span),
            StmtKind::Break(label) => self.breaks.push((stmt.span, label.is_some())),
            StmtKind::Continue(label) => self.continues.push((stmt.span, label.is_some())),
            StmtKind::Return(_) => self.returns.push(stmt.span),
            _ => {}
        }
    }

    fn expr(&mut self, expr: &Expr) {
        if let ExprKind::Lambda { .. } = expr.kind {
            self.calls.push(expr.span);
        }
    }
}

pub(super) fn span_range(span: Span) -> std::ops::Range<usize> {
    span.start as usize..span.end as usize
}

/// The fns a call of which reads the same values wherever it is, so the
/// call reads like `read<T>()` in any expression: no parameters, a body
/// whose only reads are `let name: T = read()` lines with types that use
/// no name the body binds, and no `return` but a last one.
/// Each name declared once in the file, as `var a: [I]T = ...` or
/// `var a: List<T> = ...`, with `T`
/// and the base its spans are relative to. A name declared twice is left
/// out, since which one a write means depends on scope.
pub(super) fn elems(items: &[(u32, cpl_syntax::ParsedItem)]) -> HashMap<String, (u32, Type)> {
    fn walk(base: u32, stmts: &[Stmt], seen: &mut HashMap<String, Option<(u32, Type)>>) {
        for stmt in stmts {
            match &stmt.kind {
                StmtKind::Let { pat, ty, .. } => {
                    let mut names = HashSet::new();
                    bind_names(pat, &mut names);
                    let elem = match (&pat.kind, ty.as_ref().map(|ty| &ty.kind)) {
                        (PatternKind::Name(_), Some(TypeKind::Array { elem, .. })) => {
                            Some((base, (**elem).clone()))
                        }
                        (PatternKind::Name(_), Some(TypeKind::Named { name, args }))
                            if name.text == "List" && args.len() == 1 =>
                        {
                            Some((base, args[0].clone()))
                        }
                        _ => None,
                    };
                    for name in names {
                        match seen.entry(name) {
                            std::collections::hash_map::Entry::Vacant(slot) => {
                                slot.insert(elem.clone());
                            }
                            std::collections::hash_map::Entry::Occupied(mut slot) => {
                                slot.insert(None);
                            }
                        }
                    }
                }
                StmtKind::For { body, .. } | StmtKind::While { body, .. } => {
                    walk(base, &body.stmts, seen)
                }
                StmtKind::Fn(decl) => {
                    if let FnBody::Block(block) = &decl.body {
                        walk(base, &block.stmts, seen)
                    }
                }
                _ => {}
            }
        }
    }
    let mut seen = HashMap::new();
    for (base, item) in items {
        walk(*base, &item.stmts, &mut seen);
    }
    seen.into_iter()
        .filter_map(|(name, elem)| Some((name, elem?)))
        .collect()
}

pub(super) fn values(
    src: &str,
    items: &[(u32, cpl_syntax::ParsedItem)],
) -> HashMap<String, (u32, Vec<Type>)> {
    let mut values = HashMap::new();
    for (base, item) in items {
        for stmt in &item.stmts {
            let StmtKind::Fn(decl) = &stmt.kind else {
                continue;
            };
            let FnBody::Block(block) = &decl.body else {
                continue;
            };
            if !decl.params.is_empty() || !decl.generics.is_empty() || !stmt.decorators.is_empty() {
                continue;
            }
            let code = |stmt: &Stmt| &src[span_range(stmt.span.shift(*base))];
            let returns = |stmt: &Stmt| {
                cpl_syntax::lex(code(stmt))
                    .tokens
                    .iter()
                    .any(|t| t.kind == cpl_syntax::TokenKind::Return)
            };
            let mut types = Vec::new();
            let mut bound = HashSet::new();
            let last = block.stmts.len().saturating_sub(1);
            let fits = block.stmts.iter().enumerate().all(|(at, stmt)| {
                if returns(stmt) && !(at == last && matches!(stmt.kind, StmtKind::Return(_))) {
                    return false;
                }
                if let StmtKind::Let {
                    pat,
                    ty,
                    init: Some(init),
                    ..
                } = &stmt.kind
                {
                    if let Some((ty, None)) = typed_read(ty.as_ref(), init) {
                        let mut used = HashSet::new();
                        Sizes(&mut used).of_type(&ty);
                        if !used.is_disjoint(&bound) {
                            return false;
                        }
                        types.push(ty.into_owned());
                        bind_names(pat, &mut bound);
                        return true;
                    }
                    bind_names(pat, &mut bound);
                }
                !reads_inside(code(stmt))
            });
            if fits && !types.is_empty() {
                values.insert(decl.name.text.clone(), (*base, types));
            }
        }
    }
    values
}

/// What each bare `read()` and `ask(..)` in `src` reads, by its file
/// span: the type checker facts give it from where its value goes
/// (R10.2.1), written as `let _: T = read()` for [`Builder::shape`].
pub(super) fn read_types(src: &str) -> HashMap<Span, String> {
    let mut db = cpl_check::Session::new();
    db.set_file("input.cp", src);
    let Some(facts) = db.facts("input.cp") else {
        return HashMap::new();
    };
    let whole = Span::new(0, src.len() as u32);
    facts
        .types_in(whole)
        .filter(|(span, ty)| {
            let text = &src[span_range(*span)];
            let called = |name: &str| {
                text.strip_prefix(name)
                    .is_some_and(|rest| rest.trim_start().starts_with('('))
            };
            (text == "read()" || called("ask")) && !ty.is_vague() && **ty != cpl_check::Ty::Error
        })
        .map(|(span, ty)| (span, format!("let _: {ty} = read()")))
        .collect()
}

/// Each `fn f(..) -> T = body` whose body reads, which reads for its
/// caller wherever it is called: a call inlines its arguments' reads,
/// then the body's.
pub(super) fn bodies<'a>(
    src: &str,
    items: &'a [(u32, cpl_syntax::ParsedItem)],
) -> HashMap<&'a str, (u32, &'a Expr)> {
    let mut bodies = HashMap::new();
    for (base, item) in items {
        for stmt in &item.stmts {
            let StmtKind::Fn(decl) = &stmt.kind else {
                continue;
            };
            let FnBody::Expr(body) = &decl.body else {
                continue;
            };
            let code = &src[span_range(body.span.shift(*base))];
            let tokens = cpl_syntax::lex(code).tokens;
            let asks = (0..tokens.len()).any(|at| {
                tokens[at].kind == cpl_syntax::TokenKind::Ident
                    && &code[span_range(tokens[at].span)] == "ask"
                    && (at == 0 || tokens[at - 1].kind != cpl_syntax::TokenKind::Dot)
            });
            if decl.generics.is_empty()
                && stmt.decorators.is_empty()
                && (reads_inside(code) || asks)
            {
                bodies.insert(decl.name.text.as_str(), (*base, body));
            }
        }
    }
    bodies
}

/// Each `fn f() -> yields T` that reads in a `while true:` loop, its last
/// statement and the only one that reads: the loop, which [`Builder::stream`]
/// reads until its sentinel `return`.
pub(super) fn streams<'a>(
    src: &str,
    items: &'a [(u32, cpl_syntax::ParsedItem)],
) -> HashMap<&'a str, (u32, &'a Stmt)> {
    let mut streams = HashMap::new();
    for (base, item) in items {
        for stmt in &item.stmts {
            let StmtKind::Fn(decl) = &stmt.kind else {
                continue;
            };
            let (FnBody::Block(block), Some(TypeKind::Yields(_))) =
                (&decl.body, decl.ret.as_ref().map(|ty| &ty.kind))
            else {
                continue;
            };
            let code = |stmt: &Stmt| &src[span_range(stmt.span.shift(*base))];
            let Some((looped, before)) = block.stmts.split_last() else {
                continue;
            };
            if decl.params.is_empty()
                && decl.generics.is_empty()
                && stmt.decorators.is_empty()
                && matches!(looped.kind, StmtKind::While { .. })
                && reads_inside(code(looped))
                && !before.iter().any(|stmt| reads_inside(code(stmt)))
            {
                streams.insert(decl.name.text.as_str(), (*base, looped));
            }
        }
    }
    streams
}

/// The fns whose reads are their callers': no parameters, no `return`, and
/// every call a whole top-level statement or a `let`'s whole value, so each
/// call reads the body's input right there, in order.
pub(super) fn readers<'a>(
    src: &str,
    items: &'a [(u32, cpl_syntax::ParsedItem)],
) -> HashMap<&'a str, (u32, &'a [Stmt])> {
    let mut readers = HashMap::new();
    for (base, item) in items {
        for stmt in &item.stmts {
            let StmtKind::Fn(decl) = &stmt.kind else {
                continue;
            };
            let FnBody::Block(block) = &decl.body else {
                continue;
            };
            let code = |stmt: &Stmt| &src[span_range(stmt.span.shift(*base))];
            // A `return` after the last read ends the fn as its end
            // would; one before it leaves later reads to the caller.
            let last = block
                .stmts
                .iter()
                .rposition(|stmt| reads_inside(code(stmt)));
            let early = block.stmts.iter().enumerate().any(|(at, stmt)| {
                let jumps = Jumps::of(stmt);
                Some(at) <= last
                    && jumps
                        .returns
                        .iter()
                        .any(|&r| !Jumps::within(&jumps.calls, r))
            });
            if decl.params.is_empty()
                && decl.generics.is_empty()
                && stmt.decorators.is_empty()
                && reads_inside(code(stmt))
                && !early
            {
                readers.insert(decl.name.text.as_str(), (*base, &block.stmts[..]));
            }
        }
    }
    // A call anywhere else (in a branch, a loop or an argument) reads
    // only some of the time, so its fn stays refused.
    let mut sites: HashMap<&str, HashSet<u32>> = HashMap::new();
    let mut seen = Vec::new();
    for (base, item) in items {
        for stmt in &item.stmts {
            let body = match &stmt.kind {
                StmtKind::Fn(decl)
                    if stmt
                        .decorators
                        .iter()
                        .any(|d| !d.directive && d.name.text == "multitest") =>
                {
                    match &decl.body {
                        FnBody::Block(block) => &block.stmts[..],
                        _ => &[],
                    }
                }
                _ => &[],
            };
            for stmt in std::iter::once(stmt).chain(body) {
                seen.push((*base, stmt));
            }
        }
    }
    for (base, stmts) in readers.values() {
        seen.extend(stmts.iter().map(|stmt| (*base, stmt)));
    }
    for (base, stmt) in seen {
        if let Some(name) = reader_call(stmt) {
            sites
                .entry(name)
                .or_default()
                .insert(base + stmt.span.start);
        }
    }
    let lexed = cpl_syntax::lex(src).tokens;
    readers.retain(|name, _| {
        let calls = lexed
            .windows(2)
            .filter(|pair| {
                pair[0].kind == cpl_syntax::TokenKind::Ident
                    && pair[1].kind == cpl_syntax::TokenKind::LParen
                    && &src[pair[0].span.start as usize..pair[0].span.end as usize] == *name
            })
            .count();
        // One of them is the fn's own `fn name(`.
        sites.get(name).map_or(0, HashSet::len) + 1 == calls
    });
    readers
}

/// The fn `stmt` calls with no arguments, as all it does or as all of a
/// `let`'s value: `f()` or `let x = f()`.
pub(super) fn reader_call(stmt: &Stmt) -> Option<&str> {
    let expr = match &stmt.kind {
        StmtKind::Expr(expr) => expr,
        StmtKind::Let {
            init: Some(init), ..
        } => init,
        _ => return None,
    };
    match &expr.kind {
        ExprKind::Call { callee, args } if args.is_empty() => match &callee.kind {
            ExprKind::Name(name) => Some(name.as_str()),
            _ => None,
        },
        _ => None,
    }
}

/// `stmt` with each reader call replaced by the reader's body, read
/// where the call is; a reader that calls itself is left to be refused.
pub(super) fn expand<'a>(
    base: u32,
    stmt: &'a Stmt,
    readers: &HashMap<&str, (u32, &'a [Stmt])>,
    depth: usize,
    out: &mut Vec<(u32, &'a Stmt)>,
) {
    match reader_call(stmt).and_then(|name| readers.get(name)) {
        Some(&(body_base, body)) if depth < 16 => {
            for inner in body {
                expand(body_base, inner, readers, depth + 1, out);
            }
        }
        // A `with` block's lines are read where it is written.
        _ => match &stmt.kind {
            StmtKind::With {
                body: Some(body), ..
            } if depth < 16 => {
                for inner in &body.stmts {
                    expand(base, inner, readers, depth + 1, out);
                }
            }
            _ => out.push((base, stmt)),
        },
    }
}

/// True when `code` calls `read()`, or `read` with named options such
/// as `read(gen: f)`, outside `//` comments.
pub(super) fn reads_inside(code: &str) -> bool {
    read_calls(code).next().is_some()
}

/// True when `code` calls `read` with named options (`gen:`, `none:`).
pub(super) fn reads_with_options(code: &str) -> bool {
    read_calls(code).any(|options| options)
}

/// Each call of the built-in `read` in `code`: whether it has options.
pub(super) fn read_calls(code: &str) -> impl Iterator<Item = bool> + '_ {
    code.lines().flat_map(|line| {
        let line = line.split("//").next().unwrap_or("");
        line.match_indices("read").filter_map(move |(at, _)| {
            let before = line[..at].chars().next_back();
            if before.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.') {
                return None;
            }
            let rest = &line[at + "read".len()..];
            // `read<T>()` and `read::<T>()` read too, never with options.
            if rest.starts_with('<') || rest.starts_with("::<") {
                return Some(false);
            }
            let rest = rest.strip_prefix('(')?.trim_start();
            if rest.starts_with(')') {
                return Some(false);
            }
            let name = rest
                .find(|c: char| !(c.is_alphanumeric() || c == '_'))
                .map_or(rest, |end| &rest[..end]);
            (!name.is_empty() && rest[name.len()..].trim_start().starts_with(':')).then_some(true)
        })
    })
}

/// `[read() for _ in lo..hi]`: the count `hi - lo` of values it reads.
pub(super) fn read_count(expr: &Expr) -> Option<Expr> {
    let ExprKind::Comprehension { body, clauses } = &expr.kind else {
        return None;
    };
    let [clause] = clauses.as_slice() else {
        return None;
    };
    let ExprKind::Range {
        lo: Some(lo),
        hi: Some(hi),
        inclusive,
        step: None,
    } = &clause.iter.kind
    else {
        return None;
    };
    if !is_read(body) || !clause.conds.is_empty() {
        return None;
    }
    let (lo, hi) = Builder::inclusive(lo, hi, *inclusive);
    Some(binary(BinOp::Add, binary(BinOp::Sub, hi, lo), int(1)))
}

/// The range of `read(1..=n)`: a size read from the input, as in
/// `domain V = read(1..=2e5)`.
pub(super) fn read_size(expr: &Expr) -> Option<(&Expr, &Expr, bool)> {
    let ExprKind::Call { callee, args } = &expr.kind else {
        return None;
    };
    let [arg] = args.as_slice() else {
        return None;
    };
    if !matches!(&callee.kind, ExprKind::Name(name) if name == "read") || arg.name.is_some() {
        return None;
    }
    match &arg.value.kind {
        ExprKind::Range {
            lo: Some(lo),
            hi: Some(hi),
            inclusive,
            step: None,
        } => Some((lo, hi, *inclusive)),
        _ => None,
    }
}

/// A domain of `count` positions, `0..count`.
pub(super) fn counted(count: &str) -> (Expr, Expr) {
    let count = Expr {
        kind: ExprKind::Name(count.into()),
        span: Span::new(0, 0),
    };
    (int(0), binary(BinOp::Sub, count, int(1)))
}

/// `read()` as `Some(None)`, and `read(none: x)` with a number `x`
/// (or a string that is one) as `Some(Some(x))`.
pub(super) fn read_none(expr: &Expr) -> Option<Option<i128>> {
    if is_read(expr) {
        return Some(None);
    }
    let ExprKind::Call { callee, args } = &expr.kind else {
        return None;
    };
    let [arg] = args.as_slice() else {
        return None;
    };
    if !matches!(&callee.kind, ExprKind::Name(name) if name == "read")
        || arg.name.as_ref().is_none_or(|name| name.text != "none")
    {
        return None;
    }
    let none = match &arg.value.kind {
        ExprKind::Str(pieces) => match pieces.as_slice() {
            [cpl_syntax::ast::StrPiece::Text(text)] => text.trim().parse().ok(),
            _ => None,
        },
        _ => eval(&arg.value, &Env::new()),
    };
    Some(Some(none?))
}

/// The `k` of `read(base: k)` or `read<T>(base: k)`.
pub(super) fn read_base(expr: &Expr) -> Option<i128> {
    let ExprKind::Call { args, .. } = &expr.kind else {
        return None;
    };
    let arg = args
        .iter()
        .find(|arg| arg.name.as_ref().is_some_and(|name| name.text == "base"))?;
    eval(&arg.value, &Env::new())
}

/// The `with io(base: k)` blocks among `stmts`, and the rest of the block
/// after a bare one: where each starts and ends, and its `k`.
pub(super) fn io_bases(base: u32, stmts: &[Stmt], out: &mut Vec<(Span, i128)>) {
    for stmt in stmts {
        let StmtKind::With { ctx, args, body } = &stmt.kind else {
            continue;
        };
        let k = args
            .iter()
            .find(|arg| arg.name.as_ref().is_some_and(|name| name.text == "base"))
            .and_then(|arg| eval(&arg.value, &Env::new()));
        let span = stmt.span.shift(base);
        match (ctx.text == "io", k, body) {
            (true, Some(k), Some(body)) => {
                out.push((span, k));
                io_bases(base, &body.stmts, out);
            }
            (true, Some(k), None) => out.push((Span::new(span.end, u32::MAX), k)),
            (_, _, Some(body)) => io_bases(base, &body.stmts, out),
            _ => {}
        }
    }
}

/// The fn `f` of `read(gen: f)`.
pub(super) fn read_gen(expr: &Expr) -> Option<String> {
    let ExprKind::Call { callee, args } = &expr.kind else {
        return None;
    };
    let [arg] = args.as_slice() else {
        return None;
    };
    match (&callee.kind, &arg.name, &arg.value.kind) {
        (ExprKind::Name(read), Some(name), ExprKind::Name(func))
            if read == "read" && name.text == "gen" =>
        {
            Some(func.clone())
        }
        _ => None,
    }
}

/// `src` lowered to print what `func()` makes after reading the input
/// above `line`. `None` when it does not build.
/// The structs `use std.m.{X}` brings in, or a stdlib name used bare, whose fields are all `Int`,
/// `Bool` or `Char`, shaped as a declared field of that type would be.
pub(super) fn std_structs(
    module: &[String],
    names: &[&str],
    glob: bool,
) -> Vec<(String, Vec<Shape>)> {
    use cpl_check::Loader;
    let Some((_, text)) = cpl_check::FsLoader.load("", module) else {
        return Vec::new();
    };
    let field = |ty: &Type| match &ty.kind {
        TypeKind::Named { name, args } if args.is_empty() => match name.text.as_str() {
            "Int" => Some(Shape::Int {
                lo: int(DEFAULT_INT.0),
                hi: int(DEFAULT_INT.1),
            }),
            "Bool" => Some(Shape::Bool),
            "Char" => Some(Shape::Char { lo: b'a', hi: b'z' }),
            _ => None,
        },
        _ => None,
    };
    let parsed = cpl_syntax::parse_file(&text);
    let mut out = Vec::new();
    for (_, item) in &parsed.items {
        for stmt in &item.stmts {
            let StmtKind::Struct(decl) = &stmt.kind else {
                continue;
            };
            let wanted = glob || names.contains(&decl.name.text.as_str());
            if !wanted || !decl.generics.is_empty() {
                continue;
            }
            if let Some(parts) = decl.fields.iter().map(|f| field(&f.ty)).collect() {
                out.push((decl.name.text.clone(), parts));
            }
        }
    }
    out
}

pub(super) fn gen_program(src: &str, func: &str, line: u32) -> Option<cpl_ir::Program> {
    let name = "input.cp";
    let mut db = cpl_check::Session::new();
    db.set_file(name, src);
    let analysis = db.analyze(name)?;
    let is_error = |d: &cpl_diag::Diagnostic| d.severity == cpl_diag::Severity::Error;
    if analysis.diagnostics.iter().any(is_error) {
        return None;
    }
    cpl_ir::lower_gen_read(&analysis, func, line).ok()
}

/// The type a `let` that reads gives its value, and the `none` of a
/// `read(none: x)`: the `T` of `read<T>()` or `read::<T>()`, else the
/// written `ty` of a `read()`; `[D] _ => read<T>()` is a `[D]T`.
pub(super) fn typed_read<'a>(
    ty: Option<&'a Type>,
    init: &'a Expr,
) -> Option<(std::borrow::Cow<'a, Type>, Option<i128>)> {
    use std::borrow::Cow;
    fn generic(expr: &Expr) -> Option<(&Type, Option<i128>)> {
        let ExprKind::Call { callee, args } = &expr.kind else {
            return None;
        };
        let ExprKind::Generic { base, args: tys } = &callee.kind else {
            return None;
        };
        let mut none = None;
        for arg in args {
            match arg.name.as_ref().map(|name| name.text.as_str()) {
                Some("base") => {}
                Some("none") => none = Some(eval(&arg.value, &Env::new())?),
                _ => return None,
            }
        }
        match (&base.kind, tys.as_slice()) {
            (ExprKind::Name(name), [ty]) if name == "read" => Some((ty, none)),
            _ => None,
        }
    }
    if let Some((read, none)) = generic(init) {
        return Some((Cow::Borrowed(read), none));
    }
    if let Some((lo, hi, inclusive)) = read_size(init) {
        let range = TypeKind::Range {
            lo: Some(Box::new(lo.clone())),
            hi: Some(Box::new(hi.clone())),
            inclusive,
        };
        return Some((
            Cow::Owned(Type {
                kind: range,
                span: init.span,
            }),
            None,
        ));
    }
    if let ExprKind::Table {
        domains,
        params,
        body,
    } = &init.kind
    {
        let elem = generic(body).map(|(ty, _)| Cow::Borrowed(ty)).or_else(|| {
            let elem = ty.and_then(|ty| match &ty.kind {
                TypeKind::Array { index, elem } if index.len() == domains.len() => Some(elem),
                _ => None,
            })?;
            is_read(body).then_some(Cow::Borrowed(&**elem))
        })?;
        let wild = |pat: &Pattern| matches!(pat.kind, PatternKind::Wildcard);
        if !params.iter().all(wild) {
            return None;
        }
        return Some((
            Cow::Owned(Type {
                kind: TypeKind::Array {
                    index: domains.clone(),
                    elem: Box::new(elem.into_owned()),
                },
                span: init.span,
            }),
            None,
        ));
    }
    if let ExprKind::Comprehension { body, clauses } = &init.kind {
        // `let a: List<T> = [read() for _ in lo..hi]` reads `[lo..hi]T`.
        let elem = ty.and_then(|ty| match &ty.kind {
            TypeKind::Named { name, args } if name.text == "List" && args.len() == 1 => {
                Some(&args[0])
            }
            _ => None,
        })?;
        let [clause] = clauses.as_slice() else {
            return None;
        };
        let ExprKind::Range {
            lo: Some(lo),
            hi: Some(hi),
            inclusive,
            step: None,
        } = &clause.iter.kind
        else {
            return None;
        };
        if !is_read(body)
            || !clause.conds.is_empty()
            || !matches!(clause.pat.kind, PatternKind::Wildcard)
        {
            return None;
        }
        return Some((
            Cow::Owned(Type {
                kind: TypeKind::Array {
                    index: vec![Type {
                        kind: TypeKind::Range {
                            lo: Some(lo.clone()),
                            hi: Some(hi.clone()),
                            inclusive: *inclusive,
                        },
                        span: clause.iter.span,
                    }],
                    elem: Box::new(elem.clone()),
                },
                span: init.span,
            }),
            None,
        ));
    }
    Some((Cow::Borrowed(ty?), read_none(init)?))
}

pub(super) fn is_read(expr: &Expr) -> bool {
    matches!(&expr.kind, ExprKind::Call { callee, args }
        if args.is_empty() && matches!(&callee.kind, ExprKind::Name(name) if name == "read"))
}

/// `let (domain D, ..) = compress([..])` of constant values: `D` and how
/// many distinct values it has.
pub(super) fn compressed(pat: &Pattern, init: &Expr, env: &super::Env) -> Option<(String, i128)> {
    let PatternKind::Tuple(pats) = &pat.kind else {
        return None;
    };
    let PatternKind::Domain(name) = &pats.first()?.kind else {
        return None;
    };
    let ExprKind::Call { callee, args } = &init.kind else {
        return None;
    };
    let (ExprKind::Name(func), [arg]) = (&callee.kind, args.as_slice()) else {
        return None;
    };
    let ExprKind::List(items) = &arg.value.kind else {
        return None;
    };
    if func != "compress" || arg.name.is_some() {
        return None;
    }
    let values: std::collections::BTreeSet<i128> = items
        .iter()
        .map(|item| super::eval(item, env))
        .collect::<Option<_>>()?;
    (!values.is_empty()).then(|| (name.text.clone(), values.len() as i128))
}
