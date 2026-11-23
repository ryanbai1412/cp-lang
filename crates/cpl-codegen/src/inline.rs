//! A loop over a call to a fn that is one list literal, `for (from, to, w)
//! in roles(a, b)` with `fn roles(a, b) = [(start, low, 2 * b), ..]`,
//! walks the literal with the arguments put in: the loop over a literal
//! keeps it on the stack, where the call built a `std::vector` each time.
//! Taken when every argument is a variable or a constant, so evaluating it
//! once per element is the same as once. A fn that is a comprehension over
//! a short constant range, `fn all() = [Dir4(k) for k in 0..4]`, is that
//! literal written out.

use crate::emit::{pad, Emitter};
use cpl_ir::{Arg, Expr, ExprKind, Iter, Mutation, PlaceKind, Stmt, Ty};

/// The most elements a constant-range comprehension is written out to.
const MAX_UNROLLED: i64 = 16;

impl<'p> Emitter<'p> {
    /// The literal `seq` evaluates to, when it calls a list-literal fn.
    pub(crate) fn inline_list_call(&self, seq: &Expr) -> Option<Expr> {
        let ExprKind::Call(f, args) = &seq.kind else {
            return None;
        };
        let func = self.program.fns.get(*f)?;
        let [Stmt::Return(Some(body))] = func.body.stmts.as_slice() else {
            return None;
        };
        let args = args
            .iter()
            .map(|a| match a {
                Arg::Value(e) if simple(e) => Some(e),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?;
        (args.len() == func.params.len()).then_some(())?;
        let param = |slot: usize| args.get(slot).map(|a| (*a).clone());
        match (&body.kind, &body.ty) {
            (ExprKind::Tuple(_), Ty::List(_)) => put_in(body, &param),
            (ExprKind::Do(block), Ty::List(_)) => unrolled(block, body, &param),
            _ => None,
        }
    }
    /// The declaration of `src` holding what `extend` appends: a list
    /// literal goes in a stack array, where a `std::vector` would allocate.
    pub(crate) fn extend_source(item: &Expr, src: &str, arg: &str, indent: usize) -> String {
        let literal = matches!((&item.kind, &item.ty), (ExprKind::Tuple(_), Ty::List(_)));
        let split = arg
            .strip_prefix("std::vector<")
            .filter(|rest| literal && rest.ends_with('}'))
            .and_then(|rest| rest.find(">{").map(|k| (&rest[..k], &rest[k + 1..])));
        match split {
            Some((cell, items)) => format!("{}const {cell} {src}[] = {items};\n", pad(indent)),
            _ => format!("{}auto {src} = {arg};\n", pad(indent)),
        }
    }
}

fn simple(e: &Expr) -> bool {
    matches!(
        e.kind,
        ExprKind::Local(_)
            | ExprKind::Global(_)
            | ExprKind::Int(_)
            | ExprKind::Bool(_)
            | ExprKind::Char(_)
            | ExprKind::Unit
    )
}

/// `[elem for k in lo..hi]` with constant bounds, as the literal of each
/// `elem` with `k` put in.
fn unrolled(
    block: &cpl_ir::Block,
    list: &Expr,
    param: &dyn Fn(usize) -> Option<Expr>,
) -> Option<Expr> {
    let [Stmt::Let(acc, Some(init)), Stmt::For(_, k, iter, each), Stmt::Give(given)] =
        block.stmts.as_slice()
    else {
        return None;
    };
    let Iter::Range {
        lo,
        hi,
        step: None,
        rev: false,
        closed,
    } = iter
    else {
        return None;
    };
    let (ExprKind::Int(lo), ExprKind::Int(hi)) = (&lo.kind, &hi.kind) else {
        return None;
    };
    let hi = hi.checked_add(i64::from(*closed))?;
    let [Stmt::Expr(Expr {
        kind: ExprKind::Mutate(Mutation::Push, place, pushed),
        ..
    })] = each.stmts.as_slice()
    else {
        return None;
    };
    let [elem] = pushed.as_slice() else {
        return None;
    };
    if !matches!(&init.kind, ExprKind::Tuple(items) if items.is_empty())
        || !matches!(given.kind, ExprKind::Local(g) if g == *acc)
        || place.kind != PlaceKind::Local(*acc)
        || hi
            .checked_sub(*lo)
            .is_none_or(|n| !(0..=MAX_UNROLLED).contains(&n))
    {
        return None;
    }
    let items = (*lo..hi)
        .map(|i| {
            put_in(elem, &|slot| {
                if slot == *k {
                    Some(Expr {
                        kind: ExprKind::Int(i),
                        ty: Ty::Int,
                        span: elem.span,
                    })
                } else if slot == *acc {
                    None
                } else {
                    param(slot)
                }
            })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(Expr {
        kind: ExprKind::Tuple(items),
        ty: list.ty.clone(),
        span: list.span,
    })
}

/// `e` with each local put in by `local`; none if `local` has no value for
/// one or `e` does more than compute.
fn put_in(e: &Expr, local: &dyn Fn(usize) -> Option<Expr>) -> Option<Expr> {
    let each = |xs: &[Expr]| {
        xs.iter()
            .map(|x| put_in(x, local))
            .collect::<Option<Vec<_>>>()
    };
    let one = |x: &Expr| put_in(x, local).map(Box::new);
    let kind = match &e.kind {
        ExprKind::Local(slot) => return local(*slot),
        ExprKind::Unit
        | ExprKind::Int(_)
        | ExprKind::Float(_)
        | ExprKind::Bool(_)
        | ExprKind::Char(_)
        | ExprKind::None
        | ExprKind::Global(_) => e.kind.clone(),
        ExprKind::Some(x) => ExprKind::Some(one(x)?),
        ExprKind::Unary(op, x) => ExprKind::Unary(*op, one(x)?),
        ExprKind::Binary(op, l, r) => ExprKind::Binary(*op, one(l)?, one(r)?),
        ExprKind::Cmp(op, l, r) => ExprKind::Cmp(*op, one(l)?, one(r)?),
        ExprKind::Cast(cast, x) => ExprKind::Cast(*cast, one(x)?),
        ExprKind::Field(x, i) => ExprKind::Field(one(x)?, *i),
        ExprKind::Tuple(items) => ExprKind::Tuple(each(items)?),
        ExprKind::Variant(v, items) => ExprKind::Variant(*v, each(items)?),
        _ => return None,
    };
    Some(Expr {
        kind,
        ty: e.ty.clone(),
        span: e.span,
    })
}

#[cfg(test)]
mod tests {
    use crate::tests::cpp;

    const ROLES: &str = concat!(
        "fn roles(a: Int, b: Int) -> List<(Int, Int)> = [(0, 2 * b), (1, a + b)]\n",
        "let n: Int = read()\n",
        "var t = 0\n",
        "for k in 0..n:\n",
        "    for (x, w) in roles(k, n): t += x * w\n",
        "print(t)\n",
    );

    #[test]
    fn a_loop_over_a_list_literal_fn_walks_the_literal() {
        let code = cpp(ROLES);
        let main = &code[code.find("for (ll k").unwrap()..];
        assert!(
            !main[..main.find("print").unwrap_or(main.len())].contains("roles("),
            "{code}"
        );
        assert!(main.contains("cpl::imul(2, n)"), "{code}");
        assert!(main.contains("_Pragma(\"GCC unroll 2\") for ("), "{code}");
    }

    #[test]
    fn a_computed_argument_keeps_the_call() {
        let code = cpp(&ROLES.replace("roles(k, n)", "roles(k + 1, n)"));
        let main = &code[code.find("for (ll k").unwrap()..];
        assert!(main.contains("roles("), "{code}");
    }

    #[test]
    fn a_loop_over_a_constant_range_comprehension_fn_walks_its_literal() {
        let code = cpp(concat!(
            "struct D(k: Int):\n",
            "    pub fn all() -> List<D> = [D(k) for k in 0..4]\n",
            "let n: Int = read()\n",
            "var t = 0\n",
            "for i in 0..n:\n",
            "    for d in D.all(): t += d.k * i\n",
            "print(t)\n",
        ));
        let main = &code[code.find("for (ll i").unwrap()..];
        assert!(
            !main[..main.find("print").unwrap_or(main.len())].contains("all("),
            "{code}"
        );
    }

    #[test]
    fn a_long_comprehension_fn_keeps_the_call() {
        let code = cpp(concat!(
            "fn all() -> List<Int> = [k for k in 0..100]\n",
            "let n: Int = read()\n",
            "var t = 0\n",
            "for i in 0..n:\n",
            "    for d in all(): t += d * i\n",
            "print(t)\n",
        ));
        let main = &code[code.find("for (ll i").unwrap()..];
        assert!(main.contains("all("), "{code}");
    }

    #[test]
    fn extending_by_a_literal_keeps_it_on_the_stack() {
        let code = cpp("let n: Int = read()\nvar out: List<Int> = []\nfor v in 0..n: out.extend([v, v + 1])\nprint(out)\n");
        assert!(
            code.contains("const ll t") && !code.contains("std::vector<ll>{v"),
            "{code}"
        );
    }
}
