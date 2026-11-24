//! The globals a block writes or reads in place, which main can't keep
//! as its own locals: what the Uses table says the block touches.

use std::collections::BTreeSet;

use cpl_ir::Block;

use crate::uses::effects::Touch;

/// The globals `setup`, the fns `keep` names, or any closure reads or
/// writes: those outside `main`'s own sight.
pub(crate) fn shared(program: &cpl_ir::Program, keep: impl Fn(usize) -> bool) -> BTreeSet<usize> {
    let mut out = BTreeSet::new();
    touched(&program.setup, &mut out);
    for (id, f) in program.fns.iter().enumerate() {
        if keep(id) {
            touched(&f.body, &mut out);
        }
    }
    for c in &program.closures {
        out.extend(c.captures.iter().filter(|c| c.is_global).map(|c| c.outer));
        touched(&c.body, &mut out);
    }
    out
}

pub(crate) fn touched(block: &Block, out: &mut BTreeSet<usize>) {
    let touch = Touch::of_block(block);
    out.extend(
        touch
            .reads
            .iter()
            .chain(&touch.writes)
            .filter(|v| v.0)
            .map(|v| v.1),
    );
}
