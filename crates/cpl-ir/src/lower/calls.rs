//! The fns a lowered body calls, so each gets lowered in turn.

use super::*;

pub(super) fn collect_block_calls(block: &Block, out: &mut Vec<FnId>) {
    for stmt in &block.stmts {
        collect_stmt_calls(stmt, out);
    }
}

pub(super) fn collect_stmt_calls(stmt: &Stmt, out: &mut Vec<FnId>) {
    match stmt {
        Stmt::Let(_, init) => {
            if let Some(e) = init {
                collect_expr_calls(e, out);
            }
        }
        Stmt::Assign(place, _, value) => {
            collect_place_calls(place, out);
            collect_expr_calls(value, out);
        }
        Stmt::Expr(e) | Stmt::Give(e) => collect_expr_calls(e, out),
        Stmt::Return(value) => {
            if let Some(e) = value {
                collect_expr_calls(e, out);
            }
        }
        Stmt::If(cond, then, otherwise) => {
            collect_expr_calls(cond, out);
            collect_block_calls(then, out);
            collect_block_calls(otherwise, out);
        }
        Stmt::While(_, cond, body) => {
            collect_expr_calls(cond, out);
            collect_block_calls(body, out);
        }
        Stmt::For(_, _, iter, body) => {
            match iter {
                Iter::Range { lo, hi, step, .. } => {
                    collect_expr_calls(lo, out);
                    collect_expr_calls(hi, out);
                    if let Some(s) = step {
                        collect_expr_calls(s, out);
                    }
                }
                Iter::RangeVal(e, _) | Iter::Seq(e, _) => collect_expr_calls(e, out),
                Iter::Walk { len, nth, .. } => {
                    collect_expr_calls(len, out);
                    out.push(*nth);
                }
                Iter::TreeRange {
                    container, lo, hi, ..
                } => {
                    collect_expr_calls(container, out);
                    if let Some((e, _)) = lo {
                        collect_expr_calls(e, out);
                    }
                    if let Some((e, _)) = hi {
                        collect_expr_calls(e, out);
                    }
                }
            }
            collect_block_calls(body, out);
        }
        Stmt::Assert { cond, message, .. } => {
            collect_expr_calls(cond, out);
            if let Some(m) = message {
                collect_expr_calls(m, out);
            }
        }
        Stmt::DefDomain { lo, len, .. } => {
            collect_expr_calls(lo, out);
            collect_expr_calls(len, out);
        }
        Stmt::DefDomainFromVal { val, .. } => collect_expr_calls(val, out),
        Stmt::Break(_) | Stmt::Continue(_) | Stmt::Trap(..) => {}
    }
}

pub(super) fn collect_place_calls(place: &Place, out: &mut Vec<FnId>) {
    match &place.kind {
        PlaceKind::Local(_) | PlaceKind::Global(_) => {}
        PlaceKind::Field(base, _) | PlaceKind::Some(base) => collect_place_calls(base, out),
        PlaceKind::Index(base, indices) => {
            collect_place_calls(base, out);
            for idx in indices {
                collect_expr_calls(idx, out);
            }
        }
    }
}

pub(super) fn collect_spec_calls(spec: &Spec, out: &mut Vec<FnId>) {
    match spec {
        Spec::Plain(_) | Spec::Line => {}
        Spec::Call(call) => collect_expr_calls(call, out),
        Spec::List(each) => collect_spec_calls(each, out),
        Spec::Parse(text, inner) => {
            collect_expr_calls(text, out);
            collect_spec_calls(inner, out);
        }
        Spec::Int { lo, hi, .. } | Spec::Str { lo, hi, .. } => {
            if let Some(e) = lo {
                collect_expr_calls(e, out);
            }
            if let Some(e) = hi {
                collect_expr_calls(e, out);
            }
        }
        Spec::Fields(_, fields) => {
            for f in fields {
                collect_spec_calls(f, out);
            }
        }
        Spec::Array { dims, elem } => {
            for d in dims {
                if let DimSpec::Int(len) = d {
                    collect_expr_calls(len, out);
                }
            }
            collect_spec_calls(elem, out);
        }
        Spec::Opt(inner, absent) => {
            collect_spec_calls(inner, out);
            if let Some(absent) = absent {
                collect_expr_calls(absent, out);
            }
        }
    }
}

pub(super) fn collect_expr_calls(e: &Expr, out: &mut Vec<FnId>) {
    match &e.kind {
        ExprKind::Unit
        | ExprKind::Int(_)
        | ExprKind::Float(_)
        | ExprKind::Bool(_)
        | ExprKind::Char(_)
        | ExprKind::Str(_)
        | ExprKind::None
        | ExprKind::Local(_)
        | ExprKind::Global(_)
        | ExprKind::DomainLen(_)
        | ExprKind::DomainLo(_)
        | ExprKind::DomainVal(_)
        | ExprKind::Closure(_)
        | ExprKind::Error => {}
        ExprKind::Do(block) => collect_block_calls(block, out),
        ExprKind::Some(a)
        | ExprKind::Unary(_, a)
        | ExprKind::Cast(_, a)
        | ExprKind::Field(a, _)
        | ExprKind::IsVariant(a, _)
        | ExprKind::Payload(a, _, _)
        | ExprKind::IsSome(a)
        | ExprKind::Unwrap(a, _) => collect_expr_calls(a, out),
        ExprKind::Binary(_, a, b) | ExprKind::Cmp(_, a, b) | ExprKind::Range { lo: a, hi: b } => {
            collect_expr_calls(a, out);
            collect_expr_calls(b, out);
        }
        ExprKind::RangeCheck { value, lo, hi, .. } => {
            collect_expr_calls(value, out);
            if let Some(lo) = lo {
                collect_expr_calls(lo, out);
            }
            if let Some(hi) = hi {
                collect_expr_calls(hi, out);
            }
        }
        ExprKind::If(a, b, c) => {
            collect_expr_calls(a, out);
            collect_expr_calls(b, out);
            collect_expr_calls(c, out);
        }
        ExprKind::Index(base, indices) => {
            collect_expr_calls(base, out);
            for i in indices {
                collect_expr_calls(i, out);
            }
        }
        ExprKind::Tuple(items) | ExprKind::Variant(_, items) | ExprKind::Builtin(_, items) => {
            for i in items {
                collect_expr_calls(i, out);
            }
        }
        ExprKind::Call(fid, args) => {
            out.push(*fid);
            for a in args {
                match a {
                    Arg::Value(v) => collect_expr_calls(v, out),
                    Arg::Mut(p) => collect_place_calls(p, out),
                }
            }
        }
        ExprKind::CallExpr(callee, args) => {
            collect_expr_calls(callee, out);
            for a in args {
                match a {
                    Arg::Value(v) => collect_expr_calls(v, out),
                    Arg::Mut(p) => collect_place_calls(p, out),
                }
            }
        }
        ExprKind::MutSlice(place, lo, hi) => {
            collect_place_calls(place, out);
            collect_expr_calls(lo, out);
            collect_expr_calls(hi, out);
        }
        ExprKind::Mutate(_, place, args) => {
            collect_place_calls(place, out);
            for a in args {
                collect_expr_calls(a, out);
            }
        }
        ExprKind::Sort(place, sort_by, desc) => {
            collect_place_calls(place, out);
            if let Some(desc) = desc {
                collect_expr_calls(desc, out);
            }
            match sort_by {
                SortBy::Natural => {}
                SortBy::Key(_, key) => collect_expr_calls(key, out),
                SortBy::Fn { func, .. } => collect_expr_calls(func, out),
            }
        }
        ExprKind::Read(spec) => collect_spec_calls(spec, out),
        ExprKind::Parse(src, spec) => {
            collect_expr_calls(src, out);
            collect_spec_calls(spec, out);
        }
        ExprKind::Print(args, end) => {
            for a in args {
                collect_expr_calls(&a.value, out);
            }
            if let Some(end) = end {
                collect_expr_calls(end, out);
            }
        }
        ExprKind::Debug(_, args) => {
            for a in args {
                collect_expr_calls(&a.value, out);
            }
        }
        ExprKind::Format(pieces) => {
            for p in pieces {
                match p {
                    FmtPiece::Value(a) => collect_expr_calls(&a.value, out),
                    FmtPiece::Number(value, _) => collect_expr_calls(value, out),
                    FmtPiece::Text(_) => {}
                }
            }
        }
        ExprKind::Fill(dims, value) => {
            for d in dims {
                if let DimSpec::Int(len) = d {
                    collect_expr_calls(len, out);
                }
            }
            if let Some(v) = value {
                collect_expr_calls(v, out);
            }
        }
    }
}
