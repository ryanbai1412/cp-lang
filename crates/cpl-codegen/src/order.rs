//! The order fns are written in.

use std::collections::BTreeSet;

use cpl_ir::Program;

/// The `reachable` fns, each after every fn it calls but the ones it reaches
/// back through a cycle, so only recursion needs a declaration ahead.
pub(crate) fn callee_first(program: &Program, reachable: &BTreeSet<usize>) -> Vec<usize> {
    let mut seen = BTreeSet::new();
    let mut order = Vec::with_capacity(reachable.len());
    for &root in reachable {
        visit(program, reachable, root, &mut seen, &mut order);
    }
    order
}

fn visit(
    program: &Program,
    reachable: &BTreeSet<usize>,
    id: usize,
    seen: &mut BTreeSet<usize>,
    order: &mut Vec<usize>,
) {
    if !reachable.contains(&id) || !seen.insert(id) {
        return;
    }
    let mut callees = Vec::new();
    crate::uses::places::each_call(&program.fns[id].body, &mut |callee, _| callees.push(callee));
    for callee in callees {
        visit(program, reachable, callee, seen, order);
    }
    order.push(id);
}
