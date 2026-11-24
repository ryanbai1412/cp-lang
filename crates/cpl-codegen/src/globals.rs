//! The globals a block writes or reads in place, which main can't keep
//! as its own locals: what the Uses table says the block touches.

use std::collections::BTreeSet;

use cpl_ir::Block;

use crate::uses::effects::Touch;

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
