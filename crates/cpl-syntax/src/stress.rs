//! `@stress(kind)` above a type alias: how `cpl stress` makes and shrinks
//! the alias's values (decision 25.32). `std` writes `@stress(tree)` above
//! `pub type Tree<V>`; a user writes it above a weighted
//! `type Tree = [0..n - 1](u: V, v: V, w: W)`.

use crate::ast::{Decorator, ExprKind, Stmt, StmtKind};
use cpl_diag::{msgs, Diagnostic};

pub use cpl_next_syntax::grammar::StressKind;

/// The kind a declaration's `@stress(..)` names, if it is one.
pub fn kind_of(decorators: &[Decorator]) -> Option<StressKind> {
    decorators.iter().find(|d| is_stress(d)).and_then(kind)
}

fn is_stress(d: &Decorator) -> bool {
    !d.directive && d.name.text == "stress"
}

fn kind(d: &Decorator) -> Option<StressKind> {
    let [arg] = d.args.as_slice() else {
        return None;
    };
    match (&arg.name, &arg.value.kind) {
        (None, ExprKind::Name(word)) => StressKind::of(word),
        _ => None,
    }
}

/// The `@stress`es on `stmt` that say nothing: on anything but a type
/// alias, or with anything but one kind.
pub fn errors(stmt: &Stmt, offset: u32) -> Vec<Diagnostic> {
    let alias = matches!(stmt.kind, StmtKind::TypeAlias { .. });
    let kinds = StressKind::ROWS.map(|r| format!("`{}`", r.1)).join(", ");
    (stmt.decorators.iter().filter(|d| is_stress(d)))
        .filter_map(|d| {
            let span = d.span.shift(offset);
            if !alias {
                Some(msgs::syntax::STRESS_NOT_ON_TYPE.error(span, &[]).with_help(
                    "put it on a `type` alias, as in `@stress(tree) type T = [0..n - 1](u: V, v: V)`",
                ))
            } else if kind(d).is_none() {
                Some(
                    msgs::syntax::STRESS_KIND
                        .error(span, &[&kinds])
                        .with_help("write one bare kind, as in `@stress(tree)`"),
                )
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first(src: &str) -> Stmt {
        crate::parse_file(src).items.remove(0).1.stmts.remove(0)
    }

    #[test]
    fn a_type_alias_says_its_kind_and_nothing_else_does() {
        for (want, word, _) in StressKind::ROWS {
            let stmt = first(&format!("@stress({word})\ntype T = [3]Int\n"));
            assert_eq!(kind_of(&stmt.decorators), Some(want), "{word}");
            assert!(errors(&stmt, 0).is_empty(), "{word}");
        }
        let plain = first("type Tree = [3](u: Int, v: Int)\n");
        assert_eq!(kind_of(&plain.decorators), None);
        for bad in [
            "@stress(forest)",
            "@stress()",
            "@stress(tree, dag)",
            "@stress(kind: tree)",
            "@stress(\"tree\")",
        ] {
            let stmt = first(&format!("{bad}\ntype T = [3]Int\n"));
            assert_eq!(kind_of(&stmt.decorators), None, "{bad}");
            let found = errors(&stmt, 0);
            assert_eq!(found.len(), 1, "{bad}");
            assert_eq!(found[0].code, cpl_diag::codes::BAD_STRESS, "{bad}");
        }
        let on_fn = first("@stress(tree)\nfn f() -> Int: 1\n");
        assert_eq!(errors(&on_fn, 0).len(), 1);
    }
}
