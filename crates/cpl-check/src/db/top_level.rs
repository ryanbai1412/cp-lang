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

/// R07.2.11: a top-level `let m = xs.map(f)` that later items index more
/// than once. Gives the name, the initializer's span and the read count.
pub(super) fn map_recomputed(
    items: &[Arc<ItemSyntax>],
    index: usize,
) -> Option<(String, Span, usize)> {
    use ast::{ExprKind, PatternKind};
    use cpl_syntax::visit::{walk_stmt, Visitor};
    let StmtKind::Let {
        pat,
        init: Some(init),
        ty: None,
        ..
    } = &items[index].parsed.stmts.first()?.kind
    else {
        return None;
    };
    let PatternKind::Name(name) = &pat.kind else {
        return None;
    };
    let ExprKind::Call { callee, .. } = &crate::check::lends::unparen(init).kind else {
        return None;
    };
    if !matches!(&callee.kind, ExprKind::Field { name, .. } if name.text == "map")
        || rewalk_kind(items, index, false).is_some()
    {
        return None;
    }
    struct Reads<'a>(&'a str, usize);
    impl Visitor for Reads<'_> {
        fn expr(&mut self, e: &ast::Expr) {
            if let ExprKind::Index { base, .. } = &e.kind {
                if matches!(&base.kind, ExprKind::Name(n) if n == self.0) {
                    self.1 += 1;
                }
            }
        }
    }
    let mut reads = Reads(name, 0);
    for item in &items[index + 1..] {
        for stmt in &item.parsed.stmts {
            walk_stmt(stmt, &mut reads);
        }
    }
    (reads.1 > 1).then(|| (name.clone(), init.span, reads.1))
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
