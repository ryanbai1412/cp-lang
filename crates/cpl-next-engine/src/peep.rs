//! Jump cleanup over each compiled block, which statement-at-a-time compiling leaves
//! with jumps to jumps and jumps to the next op. A jump goes straight to where its chain
//! ends, `if !c goto next-but-one; goto t` becomes `if c goto t`, and ops nothing reaches
//! and jumps to the next op go. Every target, table and mark is renumbered with them.
use crate::compile::{Code, Op, LEAVE};

pub(crate) fn tidy(c: &mut Code) {
    for b in 0..c.blocks.len() {
        let mut ops = std::mem::take(&mut c.blocks[b]);
        loop {
            thread(&mut ops, &mut c.tables);
            let keep = kept(&ops, &c.tables);
            if keep.iter().all(|&k| k) {
                break;
            }
            renumber(&mut ops, &mut c.marks[b], &mut c.tables, &keep);
        }
        c.blocks[b] = ops;
    }
}

/// Where a jump to `t` ends up after the jumps it lands on; a cycle of jumps stays.
fn end(ops: &[Op], mut t: u32) -> u32 {
    for _ in 0..ops.len() {
        match ops.get(t as usize) {
            Some(&Op::Jump { t: u }) if u != t => t = u,
            _ => break,
        }
    }
    t
}

fn targets(op: &Op, tables: &[Vec<u32>]) -> Vec<u32> {
    match *op {
        Op::Jump { t } | Op::JumpUnless { t, .. } | Op::JumpIf { t, .. } => vec![t],
        Op::Switch { table, .. } => tables[table as usize].clone(),
        Op::Region { jumps, .. } => vec![jumps.brk, jumps.cont, jumps.other],
        _ => Vec::new(),
    }
}

fn thread(ops: &mut [Op], tables: &mut [Vec<u32>]) {
    let mut aimed = vec![false; ops.len() + 1];
    for op in ops.iter() {
        for t in targets(op, tables) {
            if let Some(a) = aimed.get_mut(t as usize) {
                *a = true;
            }
        }
    }
    for i in 0..ops.len() {
        match ops[i] {
            Op::Jump { t } => ops[i] = Op::Jump { t: end(ops, t) },
            Op::JumpIf { c, t } => ops[i] = Op::JumpIf { c, t: end(ops, t) },
            Op::JumpUnless { c, t } => {
                let t = end(ops, t);
                ops[i] = match ops.get(i + 1) {
                    Some(&Op::Jump { t: u }) if t as usize == i + 2 && !aimed[i + 1] => {
                        ops[i + 1] = Op::Jump { t };
                        Op::JumpIf { c, t: end(ops, u) }
                    }
                    _ => Op::JumpUnless { c, t },
                }
            }
            Op::Switch { table, .. } => {
                for k in 0..tables[table as usize].len() {
                    tables[table as usize][k] = end(ops, tables[table as usize][k]);
                }
            }
            Op::Region { mut jumps, .. } => {
                // `LEAVE` here hands the flow back, where a jump to `LEAVE` ends the block.
                for t in [&mut jumps.brk, &mut jumps.cont, &mut jumps.other] {
                    let e = end(ops, *t);
                    if e != LEAVE {
                        *t = e;
                    }
                }
                if let Op::Region { jumps: j, .. } = &mut ops[i] {
                    *j = jumps;
                }
            }
            _ => {}
        }
    }
}

/// The ops something reaches from the start of the block, less the jumps to the op after.
fn kept(ops: &[Op], tables: &[Vec<u32>]) -> Vec<bool> {
    let mut seen = vec![false; ops.len()];
    let mut work = vec![0usize];
    while let Some(i) = work.pop() {
        if i >= ops.len() || seen[i] {
            continue;
        }
        seen[i] = true;
        if !matches!(ops[i], Op::Jump { .. } | Op::Leave(_) | Op::Fail { .. }) {
            work.push(i + 1);
        }
        work.extend(targets(&ops[i], tables).into_iter().map(|t| t as usize));
    }
    let mut next = ops.len();
    for i in (0..ops.len()).rev() {
        if !seen[i] {
            continue;
        }
        if matches!(ops[i], Op::Jump { t } if t as usize == next) {
            seen[i] = false;
        } else {
            next = i;
        }
    }
    seen
}

fn renumber(ops: &mut Vec<Op>, marks: &mut [(u32, u32)], tables: &mut [Vec<u32>], keep: &[bool]) {
    let mut at = Vec::with_capacity(keep.len() + 1);
    let mut n = 0u32;
    for &k in keep {
        at.push(n);
        n += u32::from(k);
    }
    at.push(n);
    let to = |t: u32| if t == LEAVE { LEAVE } else { at[t as usize] };
    let mut i = 0;
    ops.retain(|_| {
        i += 1;
        keep[i - 1]
    });
    for op in ops.iter_mut() {
        match op {
            Op::Jump { t } | Op::JumpUnless { t, .. } | Op::JumpIf { t, .. } => *t = to(*t),
            Op::Switch { table, .. } => {
                for t in &mut tables[*table as usize] {
                    *t = to(*t);
                }
            }
            Op::Region { jumps, .. } => {
                jumps.brk = to(jumps.brk);
                jumps.cont = to(jumps.cont);
                jumps.other = to(jumps.other);
            }
            _ => {}
        }
    }
    for m in marks.iter_mut() {
        m.0 = at[m.0 as usize];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A loop head as statements compile it: `if !c goto 3; goto 4; goto 6; body; goto 0`.
    #[test]
    fn a_loop_head_becomes_one_conditional_jump() {
        let mut ops = vec![
            Op::JumpUnless { c: 1, t: 2 },
            Op::Jump { t: 3 },
            Op::Jump { t: 5 },
            Op::Imm { d: 2, v: 7 },
            Op::Jump { t: 0 },
            Op::Leave(crate::compile::Leave::Return),
        ];
        let mut marks = vec![(0, 0), (3, 1)];
        loop {
            thread(&mut ops, &mut []);
            let keep = kept(&ops, &[]);
            if keep.iter().all(|&k| k) {
                break;
            }
            renumber(&mut ops, &mut marks, &mut [], &keep);
        }
        let shown: Vec<String> = ops.iter().map(|o| format!("{o:?}")).collect();
        assert_eq!(
            shown,
            [
                "JumpUnless { c: 1, t: 3 }",
                "Imm { d: 2, v: 7 }",
                "Jump { t: 0 }",
                "Leave(Return)"
            ]
        );
        assert_eq!(marks, [(0, 0), (1, 1)]);
    }
}
