//! Jump cleanup over each compiled block, which statement-at-a-time compiling leaves
//! with jumps to jumps and jumps to the next op. A jump goes straight to where its chain
//! ends, `if !c goto next-but-one; goto t` becomes `if c goto t`, and ops nothing reaches
//! and jumps to the next op go. Every target, table and mark is renumbered with them.
//! A drop or retain of a value that owns nothing goes, and so does a line kept and put back
//! around ops that can neither trap nor change the line.
use crate::compile::{Code, Op, Reg, LEAVE};
use crate::layout::Layouts;

pub(crate) fn tidy(c: &mut Code) {
    for b in 0..c.blocks.len() {
        let mut ops = std::mem::take(&mut c.blocks[b]);
        let chained: Vec<Reg> = c.chains.iter().flatten().map(|&(_, save)| save).collect();
        let keep = quiet(&ops, &c.tables, &c.lays, &chained);
        renumber(&mut ops, &mut c.marks[b], &mut c.tables, &keep);
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

/// The ops worth running: not a drop or retain of a value that owns nothing, nor an
/// `Enter`/`Restore` pair with only quiet ops between and no jump into them. A trap's line
/// and call stack are read only when something traps, so a pair nothing in can trap at is
/// never seen.
/// `chained`: the saves a trap's call stack reads (`Code::chains`), kept where it can trap.
fn quiet(ops: &[Op], tables: &[Vec<u32>], lays: &Layouts, chained: &[Reg]) -> Vec<bool> {
    let mut keep = vec![true; ops.len()];
    let mut aimed = vec![false; ops.len() + 1];
    let mut edges = Vec::new();
    for (from, op) in ops.iter().enumerate() {
        for t in targets(op, tables) {
            if let Some(a) = aimed.get_mut(t as usize) {
                *a = true;
                edges.push((from, t as usize));
            }
        }
    }
    for (i, op) in ops.iter().enumerate() {
        if let Op::Drop { lay, .. } | Op::Retain { lay, .. } = *op {
            keep[i] = lays.owns[lay as usize];
        }
    }
    // Innermost pairs first, so a pair around a quiet pair goes too. A pair goes when it
    // leaves the line a trap reports as it was: what it encloses keeps the line, and the line
    // it enters is the one already set, or none, or nothing it encloses can trap.
    for i in (0..ops.len()).rev() {
        let Op::Enter { line, save } = ops[i] else {
            continue;
        };
        let mut j = i + 1;
        let mut trapless = true;
        while j < ops.len() && (!keep[j] || keeps_line(&ops[j])) {
            trapless &= !keep[j] || (still(&ops[j]) && !aimed[j]);
            j += 1;
        }
        let set = trapless
            || !chained.contains(&save)
                && (line == 0 || line_at(ops, &keep, &aimed, i) == Some(line));
        // Every way to the restore passes the enter: no jump from outside lands between.
        let closed = || {
            edges
                .iter()
                .all(|&(from, t)| !(i < t && t <= j) || (i < from && from < j))
        };
        if j < ops.len()
            && matches!(ops[j], Op::Restore { save: s } if s == save)
            && set
            && closed()
        {
            keep[i] = false;
            keep[j] = false;
        }
    }
    keep
}

/// The line set when `ops[i]` starts, if every way to it sets the same one.
fn line_at(ops: &[Op], keep: &[bool], aimed: &[bool], i: usize) -> Option<u32> {
    for k in (0..i).rev() {
        match ops[k] {
            Op::At { line } | Op::Enter { line, .. } if line != 0 && keep[k] => return Some(line),
            op if keep[k] && !keeps_line(&op) => return None,
            _ if aimed[k] => return None,
            _ => {}
        }
    }
    None
}

/// An op that leaves the line a trap reports as it is, though it may trap at it.
fn keeps_line(op: &Op) -> bool {
    !matches!(
        op,
        Op::At { .. }
            | Op::Enter { .. }
            | Op::Restore { .. }
            | Op::Call { .. }
            | Op::Region { .. }
            | Op::Yield { .. }
            | Op::LendOut { .. }
    )
}

/// An op that cannot trap or change the line a trap reports.
fn still(op: &Op) -> bool {
    matches!(
        op,
        Op::Imm { .. }
            | Op::Addr { .. }
            | Op::Move { .. }
            | Op::Zero { .. }
            | Op::GetG { .. }
            | Op::PutG { .. }
            | Op::Not { .. }
    )
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

    /// A kept line around moves goes; around a `Take`, which can trap, it stays, and a drop
    /// of a word that owns nothing goes.
    #[test]
    fn a_line_nothing_can_trap_at_is_not_kept() {
        let ops = vec![
            Op::Enter { line: 3, save: 9 },
            Op::Drop { s: 1, lay: 0 },
            Op::Enter { line: 0, save: 8 },
            Op::Move { d: 1, s: 2, n: 1 },
            Op::Restore { save: 8 },
            Op::Take { d: 1, p: 0, n: 1 },
            Op::Restore { save: 9 },
            Op::Enter { line: 0, save: 7 },
            Op::Move { d: 1, s: 2, n: 1 },
            Op::Restore { save: 7 },
        ];
        let mut lays = Layouts::default();
        lays.owns.push(false);
        let keep = quiet(&ops, &[], &lays, &[]);
        assert_eq!(
            keep,
            [true, false, false, true, false, true, true, false, true, false]
        );
    }

    /// Around a `Take`, which can trap, a pair entering no line goes, and so does one entering
    /// the line already set; one entering another line stays, as does one whose set line is
    /// reached by a jump that may set another. A jump inside a pair keeps it from going only
    /// when it comes from outside. A pair whose save a call stack reads stays.
    #[test]
    fn a_line_already_set_is_not_entered_again() {
        let ops = vec![
            Op::Enter { line: 0, save: 9 },
            Op::Take { d: 1, p: 0, n: 1 },
            Op::Restore { save: 9 },
            Op::At { line: 5 },
            Op::Enter { line: 5, save: 9 },
            Op::Take { d: 1, p: 0, n: 1 },
            Op::Restore { save: 9 },
            Op::Enter { line: 4, save: 9 },
            Op::Take { d: 1, p: 0, n: 1 },
            Op::Restore { save: 9 },
            Op::Imm { d: 1, v: 0 },
            Op::Enter { line: 4, save: 9 },
            Op::Take { d: 1, p: 0, n: 1 },
            Op::Restore { save: 9 },
            Op::JumpIf { c: 1, t: 10 },
            Op::Enter { line: 0, save: 9 },
            Op::JumpIf { c: 1, t: 18 },
            Op::Take { d: 1, p: 0, n: 1 },
            Op::Restore { save: 9 },
            Op::Enter { line: 0, save: 9 },
            Op::Take { d: 1, p: 0, n: 1 },
            Op::Restore { save: 9 },
            Op::JumpIf { c: 1, t: 21 },
        ];
        let keep = quiet(&ops, &[], &Layouts::default(), &[]);
        let kept: Vec<usize> = (0..ops.len()).filter(|&i| keep[i]).collect();
        let chained = quiet(&ops, &[], &Layouts::default(), &[9]);
        assert_eq!(chained.iter().filter(|&&k| k).count(), ops.len());
        assert_eq!(
            kept,
            [1, 3, 5, 7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 20, 21, 22]
        );
    }
}
