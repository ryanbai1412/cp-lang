//! Trait declarations: associated types and aliases.

use crate::ast::*;
use crate::*;

fn stmts(src: &str) -> Vec<Stmt> {
    let file = parse_file(src);
    assert!(file.diagnostics.is_empty(), "{:#?}", file.diagnostics);
    file.items
        .into_iter()
        .flat_map(|(_, item)| item.stmts)
        .collect()
}

#[test]
fn associated_types_take_bounds_and_aliases_bound_them() {
    let s = stmts("pub trait Next<V>:\n    type Arc: Target<V> + Show\n    fn out(self, u: V) -> yields Self.Arc\ntrait WNext<V, W> = Next<V> where Self.Arc: Weighted<W>\n");
    let StmtKind::Trait { body, .. } = &s[0].kind else {
        panic!()
    };
    assert!(matches!(&body.stmts[0].kind, StmtKind::TypeAlias { bounds, .. } if bounds.len() == 2));
    let StmtKind::Trait { alias, bounds, .. } = &s[1].kind else {
        panic!()
    };
    assert!(*alias);
    assert_eq!(bounds[0].name.text, "Self.Arc");
    assert_eq!(bounds[0].traits.len(), 1);
}
