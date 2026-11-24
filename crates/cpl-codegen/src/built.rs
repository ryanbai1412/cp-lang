//! The structs no expression builds: their fn fields hold no value, so
//! they need no `std::function` (performance/R13).

use std::collections::BTreeSet;

use cpl_ir::{ClosureId, ExprKind, FnId, Program, Ty};

use crate::walk;

/// The structs with a fn field that no expression of the program builds,
/// or of only the fns and closures `within` names (those the file writes).
/// A struct value is only ever made by a literal (every other value of it
/// is a copy of one), so none of these exists when the program runs.
pub(crate) fn unbuilt(
    program: &Program,
    within: Option<(&BTreeSet<FnId>, &BTreeSet<ClosureId>)>,
) -> BTreeSet<usize> {
    let mut unbuilt: BTreeSet<usize> = (0..program.structs.len())
        .filter(|&s| {
            program.structs[s]
                .fields
                .iter()
                .any(|f| matches!(f.1, Ty::Fn(..)))
        })
        .collect();
    let mut blocks = vec![&program.setup, &program.main];
    let fn_in = |id: &FnId| within.is_none_or(|(fns, _)| fns.contains(id));
    let closure_in = |id: &ClosureId| within.is_none_or(|(_, cs)| cs.contains(id));
    blocks.extend(
        program
            .fns
            .iter()
            .enumerate()
            .filter(|(id, _)| fn_in(id))
            .map(|(_, f)| &f.body),
    );
    blocks.extend(
        program
            .closures
            .iter()
            .enumerate()
            .filter(|(id, _)| closure_in(id))
            .map(|(_, c)| &c.body),
    );
    for block in blocks {
        walk::deep_exprs(block, &mut |e| {
            if let (ExprKind::Tuple(_), Ty::Struct(s)) = (&e.kind, &e.ty) {
                unbuilt.remove(s);
            }
        });
    }
    unbuilt
}
