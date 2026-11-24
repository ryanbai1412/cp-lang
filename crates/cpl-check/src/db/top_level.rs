//! Whole-file checks on top-level statements: oversized and padded
//! arrays, recomputed maps and re-walked generators.

use super::*;

/// A top-level array over 1 MB whose record element is more than 20%
/// padding. Gives the element type's span, its data and padded sizes, and
/// the array's bytes.
pub(super) fn padded_array(
    items: &[Arc<ItemSyntax>],
    index: usize,
) -> Option<(Span, u64, u64, f64)> {
    use ast::TypeKind;
    let item = &items[index];
    let StmtKind::Let { ty: Some(ty), .. } = &item.parsed.stmts.first()?.kind else {
        return None;
    };
    let TypeKind::Array { index: dims, elem } = &ty.kind else {
        return None;
    };
    let TypeKind::Record(fields) = &elem.kind else {
        return None;
    };
    let widths = fields
        .iter()
        .map(|(_, t)| match &t.kind {
            TypeKind::Named { name, args } if args.is_empty() => match name.text.as_str() {
                "Int" | "Float" | "U64" => Some(8),
                "Char" => Some(4),
                "Bool" => Some(1),
                _ => None,
            },
            _ => None,
        })
        .collect::<Option<Vec<u64>>>()?;
    let data: u64 = widths.iter().sum();
    let align = widths.iter().copied().max()?;
    let size = data.div_ceil(align) * align;
    let count = dims.iter().try_fold(1f64, |acc, d| {
        let text = item.text.get(d.span.start as usize..d.span.end as usize)?;
        Some(acc * text.replace('_', "").parse::<f64>().ok()?)
    })?;
    let bytes = count * size as f64;
    (bytes > 1e6 && (size - data) * 5 > size).then_some((elem.span, data, size, bytes))
}

/// R07.2.11: a `let v = xs.map(f)` or `xs.filter(p)` view, at the top level
/// or in a block, read again on every pass (L0603): indexed more than once
/// or under two loops, or walked inside a loop, comprehension or lambda.
/// Each read reruns the function, so copying it once is the fix.
pub(super) struct ViewReread {
    name: String,
    init: Span,
    method: &'static str,
    func: Option<String>,
    index_reads: usize,
    repeated: bool,
}

impl ViewReread {
    pub(super) fn diagnostic(&self, shift: u32) -> Diagnostic {
        let (name, method) = (&self.name, self.method);
        let (reruns, runs) = match &self.func {
            Some(f) => (format!("`{f}`"), format!("`{f}`")),
            None => ("its function".to_string(), "it".to_string()),
        };
        let how = if self.repeated {
            "a loop reads it again on every pass".to_string()
        } else {
            format!("it is read {} times by index", self.index_reads)
        };
        let at = self.init.shift(shift);
        let mut diag = Diagnostic::warning(
            codes::MAP_RECOMPUTED,
            at,
            format!("`{name}` is a lazy `.{method}` view and {how}: each read reruns {reruns}"),
        )
        .with_help(format!(
            "copy it once with `List(...)`, so {runs} runs once per item and each read of `{name}` is plain (R07.2.11)"
        ));
        diag.fixes.push(cpl_diag::Fix {
            title: format!("copy `{name}` once with `List(...)`"),
            edits: vec![
                (Span::new(at.start, at.start), "List(".to_string()),
                (Span::new(at.end, at.end), ")".to_string()),
            ],
        });
        diag
    }
}

/// The `.map` or `.filter` view a `let` binds without a type: its name,
/// initializer, method and function.
fn view_let(stmt: &ast::Stmt) -> Option<(&str, &ast::Expr, &'static str, &ast::Expr)> {
    use ast::{ExprKind, PatternKind};
    let StmtKind::Let {
        pat,
        init: Some(init),
        ty: None,
        ..
    } = &stmt.kind
    else {
        return None;
    };
    let PatternKind::Name(name) = &pat.kind else {
        return None;
    };
    let ExprKind::Call { callee, args } = &crate::check::lends::unparen(init).kind else {
        return None;
    };
    let ExprKind::Field { name: m, .. } = &callee.kind else {
        return None;
    };
    let method = match m.text.as_str() {
        "map" => "map",
        "filter" => "filter",
        _ => return None,
    };
    let [arg] = args.as_slice() else {
        return None;
    };
    Some((name, init, method, &arg.value))
}

/// Reads of the view `name`, in statements that come after its `let`.
/// `loops` holds the spans of the loop bodies, comprehensions and lambdas
/// seen so far (the walk is pre-order), so a read inside them repeats.
struct Reads<'a> {
    name: &'a str,
    loops: Vec<Span>,
    index_reads: usize,
    repeated: bool,
    seen: bool,
}

impl Reads<'_> {
    fn depth(&self, at: Span) -> usize {
        self.loops
            .iter()
            .filter(|s| s.start <= at.start && at.end <= s.end)
            .count()
    }

    fn is_view(&self, e: &ast::Expr) -> bool {
        matches!(&crate::check::lends::unparen(e).kind, ast::ExprKind::Name(n) if n == self.name)
    }
}

impl cpl_syntax::visit::Visitor for Reads<'_> {
    fn stmt(&mut self, s: &ast::Stmt) {
        match &s.kind {
            StmtKind::For { iter, body, .. } => {
                if self.is_view(iter) && self.depth(iter.span) > 0 {
                    self.repeated = true;
                }
                self.loops.push(body.span);
            }
            StmtKind::While { body, .. } => self.loops.push(body.span),
            _ => {}
        }
    }

    fn expr(&mut self, e: &ast::Expr) {
        use ast::ExprKind;
        match &e.kind {
            ExprKind::Lambda { .. } | ExprKind::Comprehension { .. } | ExprKind::Table { .. } => {
                self.loops.push(e.span)
            }
            ExprKind::Index { base, .. } if self.is_view(base) => {
                self.index_reads += 1;
                self.repeated |= self.depth(e.span) > 1;
            }
            ExprKind::Call { callee, .. } => {
                if let ExprKind::Field { base, .. } = &callee.kind {
                    if self.is_view(base) && self.depth(e.span) > 0 {
                        self.repeated = true;
                    }
                }
            }
            ExprKind::Name(n) if n == self.name => self.seen = true,
            _ => {}
        }
    }
}

fn reread<'a>(
    stmt: &ast::Stmt,
    text: &str,
    groups: impl Iterator<Item = &'a [ast::Stmt]>,
    stop_after_first: bool,
    is_view: &dyn Fn(Span) -> bool,
) -> Option<ViewReread> {
    let (name, init, method, func) = view_let(stmt)?;
    if !is_view(crate::check::lends::unparen(init).span) {
        return None;
    }
    let mut reads = Reads {
        name,
        loops: Vec::new(),
        index_reads: 0,
        repeated: false,
        seen: false,
    };
    for group in groups {
        reads.loops.clear();
        for s in group {
            cpl_syntax::visit::walk_stmt(s, &mut reads);
        }
        if stop_after_first && reads.seen {
            break;
        }
    }
    if !(reads.repeated || method == "map" && reads.index_reads > 1) {
        return None;
    }
    let func = text
        .get(func.span.start as usize..func.span.end as usize)
        .filter(|t| t.len() <= 32 && !t.contains('\n'))
        .map(str::to_string);
    Some(ViewReread {
        name: name.to_string(),
        init: init.span,
        method,
        func,
        index_reads: reads.index_reads,
        repeated: reads.repeated,
    })
}

/// Every [`ViewReread`] whose `let` is the top-level item at `index` (read
/// by the later items) or sits in one of its blocks. The later items are
/// walked one at a time: their spans are each item's own. A lazy chain
/// walked by two items is L0602's, so only its first walking item counts.
/// `is_view` says whether the checker typed an initializer (by its span) as
/// a sequence view: `opt.map(f)` on an optional is an eager `?U`.
pub(super) fn view_rereads(
    items: &[Arc<ItemSyntax>],
    index: usize,
    is_view: &dyn Fn(Span) -> bool,
) -> Vec<ViewReread> {
    let item = &items[index];
    let mut out = Vec::new();
    if let Some(stmt) = item.parsed.stmts.first() {
        let chain = rewalk_kind(items, index, false).is_some();
        let later = items[index + 1..].iter().map(|i| i.parsed.stmts.as_slice());
        out.extend(reread(stmt, &item.text, later, chain, is_view));
    }
    struct Blocks<'a>(&'a str, Vec<ViewReread>, &'a dyn Fn(Span) -> bool);
    impl cpl_syntax::visit::Visitor for Blocks<'_> {
        fn block(&mut self, block: &ast::Block) {
            for (k, stmt) in block.stmts.iter().enumerate() {
                let after = std::iter::once(&block.stmts[k + 1..]);
                self.1.extend(reread(stmt, self.0, after, false, self.2));
            }
        }
    }
    let mut blocks = Blocks(&item.text, Vec::new(), is_view);
    for stmt in &item.parsed.stmts {
        cpl_syntax::visit::walk_stmt(stmt, &mut blocks);
    }
    out.extend(blocks.1);
    out
}

/// What makes the top-level `let` at `index` walkable once: a lazy chain
/// (L0602) or a call of a generator with a `mut` argument (L0604): one of
/// the file's, or any whose value the checker typed as one (`gen_typed`),
/// such as `std.graph.dfs`.
pub(super) fn rewalk_kind(
    items: &[Arc<ItemSyntax>],
    index: usize,
    gen_typed: bool,
) -> Option<(cpl_diag::Code, &'static str)> {
    use ast::ExprKind;
    let stmt = items[index].parsed.stmts.first()?;
    let StmtKind::Let {
        init: Some(init),
        ty: None,
        ..
    } = &stmt.kind
    else {
        return None;
    };
    fn lazy(e: &ast::Expr) -> bool {
        let ExprKind::Call { callee, .. } = &crate::check::lends::unparen(e).kind else {
            return false;
        };
        let ExprKind::Field { base, name } = &callee.kind else {
            return false;
        };
        match name.text.as_str() {
            "filter" | "take_while" | "drop_while" | "flat_map" => true,
            "map" | "take" | "drop" | "enumerate" => lazy(base),
            _ => false,
        }
    }
    if lazy(init) {
        return Some((codes::CHAIN_WALKED_TWICE, "a lazy chain"));
    }
    let ExprKind::Call { callee, args } = &crate::check::lends::unparen(init).kind else {
        return None;
    };
    let ExprKind::Name(f) = &callee.kind else {
        return None;
    };
    if !args
        .iter()
        .any(|a| a.mutable || matches!(a.value.kind, ExprKind::Mut(_)))
    {
        return None;
    }
    let generator = gen_typed
        || items.iter().any(|item| {
            item.parsed.stmts.iter().any(|s| match &s.kind {
            StmtKind::Fn(decl) => {
                decl.name.text == *f
                    && matches!(&decl.ret, Some(t) if matches!(t.kind, ast::TypeKind::Yields(_)))
            }
            _ => false,
        })
        });
    generator.then_some((
        codes::MUT_GENERATOR_REWALKED,
        "a generator holding a `mut` lend",
    ))
}

/// R03.1.8: a top-level `var a: [n]Int` with `n >= 1e6` whose every write,
/// all in the top-level code, is an integer constant. Gives the name, the
/// type's span and the range of the written values.
pub(super) fn wide_array(
    items: &[Arc<ItemSyntax>],
    checks: &[Arc<ItemCheck>],
    index: usize,
) -> Option<(String, Span, i128, i128)> {
    use ast::{ExprKind, PatternKind, TypeKind, UnOp};
    fn constant(e: &ast::Expr) -> Option<i128> {
        match &crate::check::lends::unparen(e).kind {
            ExprKind::Int(v) => Some(*v),
            ExprKind::Unary {
                op: UnOp::Neg,
                expr,
            } => constant(expr).map(|v| -v),
            _ => None,
        }
    }
    let item = &items[index];
    let StmtKind::Let {
        mutable: true,
        pat,
        ty: Some(ty),
        init: Some(init),
        ..
    } = &item.parsed.stmts.first()?.kind
    else {
        return None;
    };
    let PatternKind::Name(name) = &pat.kind else {
        return None;
    };
    let TypeKind::Array { index: dims, elem } = &ty.kind else {
        return None;
    };
    if !matches!(&elem.kind, TypeKind::Named { name, args } if name.text == "Int" && args.is_empty())
    {
        return None;
    }
    let size = dims.iter().try_fold(1f64, |acc, d| {
        let text = item.text.get(d.span.start as usize..d.span.end as usize)?;
        Some(acc * text.replace('_', "").parse::<f64>().ok()?)
    })?;
    if size < 1e6 {
        return None;
    }
    let mut values = Vec::new();
    match &crate::check::lends::unparen(init).kind {
        ExprKind::Call { callee, args } => {
            let ExprKind::Field { name, .. } = &callee.kind else {
                return None;
            };
            let [arg] = args.as_slice() else {
                return None;
            };
            if name.text != "filled" {
                return None;
            }
            values.push(constant(&arg.value)?);
        }
        e => values.push(constant(&ast::Expr {
            kind: e.clone(),
            span: init.span,
        })?),
    }
    for (k, other) in items.iter().enumerate().skip(index + 1) {
        let uses = checks[k]
            .refs
            .iter()
            .filter(|r| r.name == *name)
            .collect::<Vec<_>>();
        if uses.is_empty() {
            continue;
        }
        if uses.iter().any(|r| !r.top_level) || other.decls.iter().any(|d| d.name == *name) {
            return None;
        }
        let stmt = other.parsed.stmts.first()?;
        match &stmt.kind {
            StmtKind::Assign {
                target,
                op: ast::AssignOp::Set,
                value,
            } => {
                let ExprKind::Index { base, .. } = &target.kind else {
                    return None;
                };
                if !matches!(&base.kind, ExprKind::Name(n) if n == name) {
                    return None;
                }
                values.push(constant(value)?);
            }
            StmtKind::Expr(e) if !matches!(&e.kind, ExprKind::Call { callee, .. } if matches!(&callee.kind, ExprKind::Field { base, .. } if matches!(&base.kind, ExprKind::Name(n) if n == name))) =>
                {}
            _ => return None,
        }
    }
    let lo = *values.iter().min()?;
    let hi = *values.iter().max()?;
    (i64::try_from(lo).is_ok() && (hi - lo) < (1 << 31)).then(|| (name.clone(), ty.span, lo, hi))
}
