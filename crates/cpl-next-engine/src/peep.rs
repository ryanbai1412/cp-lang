//! Jump cleanup over each compiled block, which statement-at-a-time compiling leaves
//! with jumps to jumps and jumps to the next op. A jump goes straight to where its chain
//! ends, `if !c goto next-but-one; goto t` becomes `if c goto t`, and ops nothing reaches
//! and jumps to the next op go. Every target, table and mark is renumbered with them.
//! An origin kept and put back around ops that can neither trap nor change the origin. A register a one-word move set is
//! read from its source while neither changes, and an op that only sets registers nothing
//! reads again goes.
use crate::compile::{Code, Kind, Op, PlaceCode, Reg, Root, Step, INHERIT, LEAVE, SAVED};
use crate::layout::{Lay, Layouts};

pub(crate) fn tidy(c: &mut Code) {
    let chained: Vec<Reg> = c.chains.iter().flatten().map(|&(_, save)| save).collect();
    // The chain registers a block reads but one outside it sets: a region body's.
    let outside: Vec<Reg> = c
        .marks
        .iter()
        .filter_map(|m| m.first())
        .flat_map(|&(_, ch)| c.chains[ch as usize].iter().map(|&(_, save)| save))
        .collect();
    let mut saved: std::collections::HashMap<Vec<u32>, u32> =
        (c.saved.iter().cloned()).zip(0..).collect();
    let addressed = addressed(c);
    // What leaving a fn reads of its frame: its result, the lent parameters it writes back
    // and the locals it drops.
    let mut leaving: Vec<Option<Vec<Words>>> = vec![None; c.blocks.len()];
    for f in &c.funcs {
        let lent = (f.params.iter().zip(&f.lent)).filter_map(|(&p, &l)| l.then_some(p));
        let drops = f.drops.iter().map(|&(r, l)| (r, c.lays.size[l as usize]));
        leaving[f.block as usize] =
            Some([(0, f.ret)].into_iter().chain(lent).chain(drops).collect());
    }
    c.origins = vec![Vec::new(); c.blocks.len()];
    for b in 0..c.blocks.len() {
        let mut ops = std::mem::take(&mut c.blocks[b]);
        let mut origins = Vec::new();
        let keep = quiet(&ops, &c.tables, &chained);
        renumber(
            &mut ops,
            &mut c.marks[b],
            &mut c.tables,
            &keep,
            &mut origins,
        );
        let fixed = fixed_origins(&ops, c, b, &outside);
        // A block whose origins are a table reads no chain register but those set outside it.
        let static_origins = fixed.is_some();
        if let Some(fixed) = fixed {
            let mut keep = vec![true; ops.len()];
            for (i, (op, f)) in ops.iter_mut().zip(&fixed).enumerate() {
                match *op {
                    Op::Enter { save, .. } if outside.contains(&save) => {
                        *op = Op::Imm {
                            d: save,
                            v: u64::from(f.after),
                        }
                    }
                    Op::At { .. } | Op::Enter { .. } | Op::Restore { .. } => keep[i] = false,
                    _ => {}
                }
            }
            for f in fixed {
                let n = c.saved.len() as u32;
                let s = *saved.entry(f.calls).or_insert_with_key(|calls| {
                    c.saved.push(calls.clone());
                    n
                });
                origins.push((f.origin, s));
            }
            renumber(
                &mut ops,
                &mut c.marks[b],
                &mut c.tables,
                &keep,
                &mut origins,
            );
        }
        let pinned = |r: Reg| {
            r >= addressed[b] || (!static_origins && chained.contains(&r)) || outside.contains(&r)
        };
        jumps(&mut ops, &mut c.marks[b], &mut c.tables, &mut origins);
        forward(&mut ops, &mut c.places, &c.lays, &c.tables, &pinned);
        let keep = unread(
            &ops,
            &c.places,
            &c.lays,
            &c.tables,
            &pinned,
            leaving[b].as_deref(),
        );
        renumber(
            &mut ops,
            &mut c.marks[b],
            &mut c.tables,
            &keep,
            &mut origins,
        );
        jumps(&mut ops, &mut c.marks[b], &mut c.tables, &mut origins);
        c.blocks[b] = ops;
        c.origins[b] = origins;
    }
}

/// Jumps go straight to where their chains end, and ops nothing reaches go.
fn jumps(
    ops: &mut Vec<Op>,
    marks: &mut [(u32, u32)],
    tables: &mut [Vec<u32>],
    origins: &mut Vec<(u32, u32)>,
) {
    loop {
        thread(ops, tables);
        let keep = kept(ops, tables);
        if keep.iter().all(|&k| k) {
            break;
        }
        renumber(ops, marks, tables, &keep, origins);
    }
}

/// The origin at an op that ways setting different origins lead to.
const MIXED: u32 = u32::MAX - 2;

/// An op that may trap or run code that does: a trap in it reports the origin it runs at. A
/// place traps only at a cell's index.
fn traps(op: &Op, places: &[PlaceCode]) -> bool {
    let indexes =
        |p: u32| (places[p as usize].steps.iter()).any(|s| matches!(s, Step::Slot { .. }));
    !still(op)
        && match *op {
            Op::Load { p, .. } | Op::Peek { p, .. } | Op::Take { p, .. } | Op::Put { p, .. } => {
                indexes(p)
            }
            _ => true,
        }
        && !matches!(
            op,
            Op::Cmp { .. }
                | Op::Jump { .. }
                | Op::JumpIf { .. }
                | Op::JumpUnless { .. }
                | Op::Switch { .. }
                | Op::Leave(_)
                | Op::Out { .. }
                | Op::ParseF64 { .. }
                | Op::Showing { .. }
                | Op::Capture
                | Op::Flush
                | Op::Event { .. }
                | Op::Text { .. }
                | Op::InputLeft { .. }
                | Op::RawEmpty { .. }
                | Op::Drop { .. }
                | Op::Retain { .. }
                | Op::At { .. }
                | Op::Enter { .. }
                | Op::Restore { .. }
        )
}

/// An op's origin in a block whose origins every way through agrees on: the origin a trap in it
/// reports, the origin after it and the call origin of each call in its chain.
struct Fixed {
    origin: u32,
    after: u32,
    calls: Vec<u32>,
}

/// The origin, and the call origins kept in chain registers, when an op starts.
type Origins = (u32, Vec<(Reg, u32)>);

/// What `op` leaves the origins as, as it sets them when run; `None` for an origin a block outside
/// keeps.
fn step(op: &Op, c: &Code, (mut origin, mut saves): Origins) -> Option<Origins> {
    match *op {
        Op::At { origin: l } => origin = l,
        Op::Enter { origin: l, save } => {
            if l != 0 {
                origin = l;
            }
            match saves.binary_search_by_key(&save, |s| s.0) {
                Ok(i) => saves[i].1 = origin,
                Err(i) => saves.insert(i, (save, origin)),
            }
        }
        Op::Restore { save } => origin = saves.iter().find(|s| s.0 == save)?.1,
        Op::Region { c: k, .. } if c.calls[k as usize].origin != 0 => {
            origin = c.calls[k as usize].origin;
        }
        _ => {}
    }
    (origin != SAVED).then_some((origin, saves))
}

/// Each op's origins when every way to it sets the same ones, so a trap can read them from a
/// table and the ops that set them go; `None` when they depend on the way taken.
fn fixed_origins(ops: &[Op], c: &Code, b: usize, outside: &[Reg]) -> Option<Vec<Fixed>> {
    let marks = &c.marks[b];
    let first = marks.first().map_or(0, |m| m.1);
    let mut start: Vec<(Reg, u32)> = c.chains[first as usize]
        .iter()
        .map(|&(_, save)| (save, SAVED))
        .collect();
    start.sort_unstable();
    let mut before: Vec<Option<Origins>> = vec![None; ops.len()];
    if let Some(b) = before.first_mut() {
        *b = Some((INHERIT, start));
    }
    let mut work = vec![0usize];
    while let Some(i) = work.pop() {
        let Some(at) = before.get(i).cloned().flatten() else {
            continue;
        };
        let after = step(&ops[i], c, at)?;
        let mut next = targets(&ops[i], &c.tables);
        if !matches!(ops[i], Op::Jump { .. } | Op::Leave(_) | Op::Fail { .. }) {
            next.push(i as u32 + 1);
        }
        for t in next
            .into_iter()
            .map(|t| t as usize)
            .filter(|&t| t < ops.len())
        {
            // Ways to an op that set different origins leave none an op that may trap can use.
            let joined = match &before[t] {
                None => after.clone(),
                Some(o) if *o == after => continue,
                Some(o) => {
                    let origin = if o.0 == after.0 { o.0 } else { MIXED };
                    let saves = o.1.iter().filter(|s| after.1.contains(s)).copied();
                    (origin, saves.collect())
                }
            };
            before[t] = Some(joined);
            work.push(t);
        }
    }
    let mut out = Vec::with_capacity(ops.len());
    for (i, op) in ops.iter().enumerate() {
        let Some(at) = before[i].clone() else {
            out.push(Fixed {
                origin: INHERIT,
                after: INHERIT,
                calls: Vec::new(),
            });
            continue;
        };
        let m = marks
            .partition_point(|m| m.0 as usize <= i)
            .saturating_sub(1);
        let chain = marks.get(m).map_or(&[][..], |m| &c.chains[m.1 as usize]);
        let calls = chain
            .iter()
            .map(|&(_, save)| at.1.iter().find(|s| s.0 == save).map(|s| s.1))
            .collect::<Option<Vec<_>>>()?;
        let (origin, after) = (at.0, step(op, c, at)?.0);
        let kept = matches!(*op, Op::Enter { save, .. } if outside.contains(&save));
        if kept && (after == INHERIT || after == MIXED)
            || traps(op, &c.places) && (origin == MIXED || calls.contains(&MIXED))
        {
            return None;
        }
        let origin = match *op {
            Op::Region { .. } => after,
            Op::Check { origin: l, .. } if l != 0 => l,
            _ => origin,
        };
        out.push(Fixed {
            origin,
            after,
            calls,
        });
    }
    Some(out)
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

/// The ops worth running: not an `Enter`/`Restore` pair with only quiet ops between and no jump into them. A trap's origin
/// and call stack are read only when something traps, so a pair nothing in can trap at is
/// never seen.
/// `chained`: the saves a trap's call stack reads (`Code::chains`), kept where it can trap.
fn quiet(ops: &[Op], tables: &[Vec<u32>], chained: &[Reg]) -> Vec<bool> {
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
    // Innermost pairs first, so a pair around a quiet pair goes too. A pair goes when it
    // leaves the origin a trap reports as it was: what it encloses keeps the origin, and the origin
    // it enters is the one already set, or none, or nothing it encloses can trap.
    for i in (0..ops.len()).rev() {
        let Op::Enter { origin, save } = ops[i] else {
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
                && (origin == 0 || origin_at(ops, &keep, &aimed, i) == Some(origin));
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

/// The origin set when `ops[i]` starts, if every way to it sets the same one.
fn origin_at(ops: &[Op], keep: &[bool], aimed: &[bool], i: usize) -> Option<u32> {
    for k in (0..i).rev() {
        match ops[k] {
            Op::At { origin } | Op::Enter { origin, .. } if origin != 0 && keep[k] => {
                return Some(origin)
            }
            op if keep[k] && !keeps_line(&op) => return None,
            _ if aimed[k] => return None,
            _ => {}
        }
    }
    None
}

/// An op that leaves the origin a trap reports as it is, though it may trap at it.
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

/// An op that cannot trap or change the origin a trap reports.
fn still(op: &Op) -> bool {
    matches!(
        op,
        Op::Imm { .. }
            | Op::Addr { .. }
            | Op::Move { .. }
            | Op::Zero { .. }
            | Op::GetG { .. }
            | Op::PutG { .. }
    ) || matches!(op, Op::Unary { op, .. } if !op.checked())
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

/// Per block, the first register of its frame whose address is taken: it, and the words
/// after it, may be read through that address. A fn's frame is its block's and that of the
/// region bodies its blocks lend to; a block in no fn's frame has every register taken.
fn addressed(c: &Code) -> Vec<Reg> {
    let mut at = vec![0; c.blocks.len()];
    for f in &c.funcs {
        let (mut group, mut todo) = (Vec::new(), vec![f.block]);
        while let Some(b) = todo.pop() {
            if group.contains(&b) {
                continue;
            }
            group.push(b);
            for op in &c.blocks[b as usize] {
                if let Op::Region { body, .. } = *op {
                    todo.push(body);
                }
            }
        }
        let first = (group.iter().flat_map(|&b| &c.blocks[b as usize]))
            .filter_map(|op| match *op {
                Op::Addr { s, .. } => Some(s),
                _ => None,
            })
            .min()
            .unwrap_or(Reg::MAX);
        for b in group {
            at[b as usize] = first;
        }
    }
    at
}

/// Words of registers: the first and how many.
type Words = (Reg, u32);

/// The registers an op reads and the ones it sets every time it runs; `None` for an op that
/// may read any (a call, a region, a yield, an event, leaving the block). A write through a
/// place to a register counts as a read.
fn effect(op: &Op, places: &[PlaceCode], lays: &Layouts) -> Option<(Vec<Words>, Vec<Words>)> {
    let size = |l: Lay| lays.size[l as usize];
    let place = |p: u32, n: u32| {
        let pc = &places[p as usize];
        let (mut v, mut off, mut through) = (Vec::new(), 0, false);
        for st in &pc.steps {
            match *st {
                Step::Off(at) if !through => off += at,
                Step::Slot { i, .. } => {
                    through = true;
                    v.push((i, 1));
                }
                _ => through = true,
            }
        }
        if let Root::Local(r) = pc.root {
            v.push((r + off, if through { 1 } else { n }));
        }
        v
    };
    let one = |r: Reg| (r, 1);
    Some(match *op {
        Op::Imm { d, .. }
        | Op::Addr { d, .. }
        | Op::Text { d, .. }
        | Op::InputLeft { d }
        | Op::ReadInt { d }
        | Op::ReadByte { d }
        | Op::RandWord { d }
        | Op::RawEmpty { d, .. } => (vec![], vec![one(d)]),
        Op::PeekByte { d, k } => (vec![one(k)], vec![one(d)]),
        Op::Enter { save, .. } => (vec![], vec![one(save)]),
        Op::Move { d, s, n } => (vec![(s, n)], vec![(d, n)]),
        Op::Int { d, a, b, .. }
        | Op::Word { d, a, b, .. }
        | Op::Float { d, a, b, .. }
        | Op::Cmp { d, a, b, .. } => (vec![one(a), one(b)], vec![one(d)]),
        Op::Wide {
            d, a, b, narrow, ..
        } => (vec![(a, 2), (b, if narrow { 1 } else { 2 })], vec![(d, 2)]),
        Op::WideCmp { d, a, b, .. } => (vec![(a, 2), (b, 2)], vec![one(d)]),
        Op::Unary { sig, d, a, .. } => (
            vec![(a, sig.args[0].words() as u32)],
            vec![(d, sig.res.words() as u32)],
        ),
        Op::RawCap { d, n: a, .. } => (vec![one(a)], vec![one(d)]),
        Op::Zero { d, n } | Op::GetG { d, n, .. } => (vec![], vec![(d, n)]),
        Op::PutG { s, n, .. } => (vec![(s, n)], vec![]),
        Op::Retain { s, lay } | Op::Drop { s, lay } => (vec![(s, size(lay))], vec![]),
        Op::Out { r, n, .. } => (vec![one(r), one(n)], vec![]),
        Op::ParseF64 { d, r, n, .. } => (vec![one(r), one(n)], vec![one(d)]),
        Op::Check { c, .. }
        | Op::Guard { c, .. }
        | Op::JumpUnless { c, .. }
        | Op::JumpIf { c, .. }
        | Op::Switch { c, .. }
        | Op::Restore { save: c } => (vec![one(c)], vec![]),
        Op::At { .. } | Op::Jump { .. } => (vec![], vec![]),
        Op::RawNew { d, n, fill, elem } => (vec![one(n), (fill, size(elem))], vec![one(d)]),
        Op::RawGet { d, r, i, elem, .. } => (vec![one(r), one(i)], vec![(d, size(elem))]),
        Op::RawSet { p, i, x, elem } => {
            let mut v = place(p, 1);
            v.extend([one(i), (x, size(elem))]);
            (v, vec![])
        }
        Op::RawTake { d, p, i, elem } => {
            let mut v = place(p, 1);
            v.push(one(i));
            (v, vec![(d, size(elem))])
        }
        Op::Load { d, p, lay } => (place(p, size(lay)), vec![(d, size(lay))]),
        Op::Peek { d, p } => (place(p, 1), vec![one(d)]),
        Op::Take { d, p, n } => (place(p, n), vec![(d, n)]),
        Op::Put { p, s, n } => {
            let mut v = place(p, n);
            v.push((s, n));
            (v, vec![])
        }
        Op::Call { .. }
        | Op::Region { .. }
        | Op::Yield { .. }
        | Op::LendOut { .. }
        | Op::Leave(_)
        | Op::Showing { .. }
        | Op::Capture
        | Op::Flush
        | Op::Event { .. }
        | Op::Fail { .. } => return None,
    })
}

fn covers(ws: &[Words], r: Reg) -> bool {
    ws.iter().any(|&(a, n)| a <= r && r < a + n)
}

/// An op that does nothing but set its registers: it can't trap, count or allocate. A
/// place read fails only on a broken engine invariant, never as a program's trap.
fn pure(op: &Op) -> bool {
    match op {
        Op::Imm { .. }
        | Op::Move { .. }
        | Op::Zero { .. }
        | Op::GetG { .. }
        | Op::Take { .. }
        | Op::Peek { .. } => true,
        Op::Unary { op, .. } => !op.checked(),
        Op::Cmp { kind, .. } => *kind != Kind::F64,
        _ => false,
    }
}

/// The one-word operands an op reads, to be renamed.
fn operands<'a>(op: &'a mut Op, lays: &Layouts) -> Vec<&'a mut Reg> {
    let one = |l: Lay| lays.size[l as usize] == 1;
    match op {
        Op::Int { a, b, .. } | Op::Word { a, b, .. } | Op::Float { a, b, .. } => vec![a, b],
        Op::Cmp { a, b, .. } => vec![a, b],
        Op::Unary { a, sig, .. } if sig.args[0].words() == 1 => vec![a],
        Op::RawCap { n: a, .. }
        | Op::Check { c: a, .. }
        | Op::Guard { c: a, .. }
        | Op::JumpUnless { c: a, .. }
        | Op::JumpIf { c: a, .. } => vec![a],
        Op::Move { s, n: 1, .. } | Op::PutG { s, n: 1, .. } | Op::Put { s, n: 1, .. } => vec![s],
        Op::RawGet { r, i, .. } => vec![r, i],
        Op::Out { r, n, .. } | Op::ParseF64 { r, n, .. } => vec![r, n],
        Op::PeekByte { k, .. } => vec![k],
        Op::RawSet { i, x, elem, .. } => match one(*elem) {
            true => vec![i, x],
            false => vec![i],
        },
        Op::RawNew { n, fill, elem, .. } => match one(*elem) {
            true => vec![n, fill],
            false => vec![n],
        },
        Op::RawTake { i, .. } => vec![i],
        _ => Vec::new(),
    }
}

/// The place an op reads or writes through.
fn place_of(op: &mut Op) -> Option<&mut u32> {
    match op {
        Op::Load { p, .. }
        | Op::Peek { p, .. }
        | Op::Take { p, .. }
        | Op::Put { p, .. }
        | Op::RawSet { p, .. }
        | Op::RawTake { p, .. } => Some(p),
        _ => None,
    }
}

/// Each op some jump lands on.
fn aimed(ops: &[Op], tables: &[Vec<u32>]) -> Vec<bool> {
    let mut aimed = vec![false; ops.len() + 1];
    for op in ops {
        for t in targets(op, tables) {
            if let Some(a) = aimed.get_mut(t as usize) {
                *a = true;
            }
        }
    }
    aimed
}

/// After `d = s` of one word, the ops that run next in an origin read `s` for `d` until either
/// is set, so the move is left with no reader when they were all its readers. A place indexed
/// by `d` becomes a new place indexed by `s`.
fn forward(
    ops: &mut [Op],
    places: &mut Vec<PlaceCode>,
    lays: &Layouts,
    tables: &[Vec<u32>],
    pinned: &dyn Fn(Reg) -> bool,
) {
    let aimed = aimed(ops, tables);
    let mut into = vec![Vec::new(); ops.len() + 1];
    for (k, op) in ops.iter().enumerate() {
        for t in targets(op, tables) {
            if let Some(v) = into.get_mut(t as usize) {
                v.push(k);
            }
        }
    }
    // A join every jump into comes from after `i` still holds what held since `i`.
    let joins = |i: usize, j: usize| !aimed[j] || into[j].iter().all(|&k| i < k && k < j);
    for i in 0..ops.len() {
        if let Op::Zero { d, n } = ops[i] {
            if !pinned(d) {
                unzero(ops, places, lays, i, d..d + n, &|j| joins(i, j));
            }
            continue;
        }
        if let Op::Move { d, s, n } = ops[i] {
            if (d + n <= s || s + n <= d) && !(d..d + n).chain(s..s + n).any(pinned) {
                roundtrip(ops, places, lays, tables, &into, i);
            }
        }
        let Op::Move { d, s, n: 1 } = ops[i] else {
            continue;
        };
        if d == s || pinned(d) {
            continue;
        }
        for (j, at) in ops.iter_mut().enumerate().skip(i + 1) {
            if !joins(i, j) {
                break;
            }
            let Some((reads, writes)) = effect(at, places, lays) else {
                break;
            };
            if covers(&reads, d) {
                let mut op = *at;
                for r in operands(&mut op, lays) {
                    if *r == d {
                        *r = s;
                    }
                }
                let made = places.len();
                if let Some(p) = place_of(&mut op) {
                    let mut pc = places[*p as usize].clone();
                    let mut renamed = false;
                    for st in &mut pc.steps {
                        if let Step::Slot { i, .. } = st {
                            if *i == d {
                                (*i, renamed) = (s, true);
                            }
                        }
                    }
                    if renamed {
                        *p = made as u32;
                        places.push(pc);
                    }
                }
                match effect(&op, places, lays) {
                    Some((reads, _)) if !covers(&reads, d) => *at = op,
                    _ => {
                        places.truncate(made);
                        break;
                    }
                }
            }
            let ends = matches!(*at, Op::Jump { .. } | Op::Switch { .. });
            if ends || covers(&writes, d) || covers(&writes, s) {
                break;
            }
        }
    }
}

/// `t = x`, ops that use `t` but never `x`, then `x = t`, with the ops entered only from the
/// first and left only into the last: the ops use `x` itself and `x = t` becomes `t = x`,
/// which leaves the first copy no reader. A path that sets `t` again sets `x`, which nothing
/// reads before `x = t`.
fn roundtrip(
    ops: &mut [Op],
    places: &mut Vec<PlaceCode>,
    lays: &Layouts,
    tables: &[Vec<u32>],
    into: &[Vec<usize>],
    i: usize,
) {
    let Op::Move { d: t, s: x, n } = ops[i] else {
        return;
    };
    let back = |op: &Op| matches!(*op, Op::Move { d, s, n: m } if (d, s, m) == (x, t, n));
    let Some(k) = (i + 1..ops.len()).find(|&k| back(&ops[k])) else {
        return;
    };
    let inside = |j: usize| i < j && j < k;
    let meets = |ws: &[Words], r: Reg| ws.iter().any(|&(a, m)| a < r + n && r < a + m);
    let mut moved = Vec::new();
    for j in i + 1..=k {
        if !into[j].iter().all(|&from| inside(from)) {
            return;
        }
        if j == k {
            break;
        }
        let op = ops[j];
        if !targets(&op, tables)
            .iter()
            .all(|&to| inside(to as usize) || to as usize == k)
        {
            return;
        }
        let Some((reads, writes)) = effect(&op, places, lays) else {
            return;
        };
        if meets(&reads, x) || meets(&writes, x) {
            return;
        }
        if meets(&reads, t) || meets(&writes, t) {
            let Some(m) = shifted(op, places, lays, t, x, n) else {
                return;
            };
            moved.push((j, m));
        }
    }
    for (j, (mut op, pc)) in moved {
        if let Some(pc) = pc {
            let made = places.len() as u32;
            places.push(pc);
            if let Some(p) = place_of(&mut op) {
                *p = made;
            }
        }
        ops[j] = op;
    }
    ops[k] = Op::Move { d: t, s: x, n };
}

/// `op` using the same word of `x..x + n` for each register of `t..t + n`, with the place
/// it goes through moved likewise; `None` where some use of `t` can't move.
fn shifted(
    op: Op,
    places: &mut Vec<PlaceCode>,
    lays: &Layouts,
    t: Reg,
    x: Reg,
    n: u32,
) -> Option<(Op, Option<PlaceCode>)> {
    let shift = |r: &mut Reg| {
        if t <= *r && *r < t + n {
            *r = *r - t + x;
        }
    };
    let mut new = op;
    match &mut new {
        Op::Move { d, s, .. } => [d, s].into_iter().for_each(shift),
        Op::Retain { s, .. } | Op::Drop { s, .. } | Op::Put { s, .. } | Op::PutG { s, .. } => {
            shift(s)
        }
        Op::RawSet { x: v, .. } | Op::RawNew { fill: v, .. } => shift(v),
        _ => {}
    }
    operands(&mut new, lays).into_iter().for_each(shift);
    if let Some(d) = dest(&mut new) {
        shift(d);
    }
    let made = places.len() as u32;
    let pc = place_of(&mut new).map(|p| {
        let mut pc = places[*p as usize].clone();
        if let Root::Local(r) = &mut pc.root {
            shift(r);
        }
        for st in &mut pc.steps {
            if let Step::Slot { i, .. } = st {
                shift(i);
            }
        }
        *p = made;
        pc
    });
    let moved = |ws: Vec<Words>| -> Option<Vec<Words>> {
        (ws.into_iter())
            .map(
                |(a, m)| match (t <= a && a + m <= t + n, a < t + n && t < a + m) {
                    (true, _) => Some((a - t + x, m)),
                    (false, true) => None,
                    (false, false) => Some((a, m)),
                },
            )
            .collect()
    };
    let (reads, writes) = effect(&op, places, lays)?;
    let want = (moved(reads)?, moved(writes)?);
    places.extend(pc.clone());
    let got = effect(&new, places, lays);
    places.truncate(made as usize);
    (got == Some(want)).then_some((new, pc))
}

/// The first register of an op's result.
fn dest(op: &mut Op) -> Option<&mut Reg> {
    match op {
        Op::Imm { d, .. }
        | Op::Text { d, .. }
        | Op::InputLeft { d }
        | Op::ReadInt { d }
        | Op::RawEmpty { d, .. }
        | Op::Int { d, .. }
        | Op::Word { d, .. }
        | Op::Float { d, .. }
        | Op::Cmp { d, .. }
        | Op::Unary { d, .. }
        | Op::RawCap { d, .. }
        | Op::RawNew { d, .. }
        | Op::RawGet { d, .. }
        | Op::RawTake { d, .. }
        | Op::Load { d, .. }
        | Op::Peek { d, .. }
        | Op::Take { d, .. }
        | Op::GetG { d, .. }
        | Op::Zero { d, .. } => Some(d),
        _ => None,
    }
}

/// A drop of registers still zero since `Zero` at `i` releases nothing: it becomes a `Zero`
/// of them, which `unread` then removes with the first when nothing reads them.
fn unzero(
    ops: &mut [Op],
    places: &[PlaceCode],
    lays: &Layouts,
    i: usize,
    zero: std::ops::Range<Reg>,
    joins: &dyn Fn(usize) -> bool,
) {
    for (j, op) in ops.iter_mut().enumerate().skip(i + 1) {
        if !joins(j) {
            break;
        }
        if let Op::Drop { s, lay } = *op {
            let n = lays.size[lay as usize];
            if zero.start <= s && s + n <= zero.end {
                *op = Op::Zero { d: s, n };
                continue;
            }
        }
        let Some((reads, writes)) = effect(op, places, lays) else {
            break;
        };
        // A store through a place rooted in the zeroed words reads them to write them.
        let through = !pure(op) && zero.clone().any(|r| covers(&reads, r));
        let ends = matches!(*op, Op::Jump { .. } | Op::Switch { .. });
        if ends || through || zero.clone().any(|r| covers(&writes, r)) {
            break;
        }
    }
}

/// The ops to keep: not a pure op whose registers nothing reads before they are set again.
/// `leaving`: the registers leaving the block reads, when it is a fn's block; a region
/// body's leaves its frame to the code that lent to it, which may read any.
fn unread(
    ops: &[Op],
    places: &[PlaceCode],
    lays: &Layouts,
    tables: &[Vec<u32>],
    pinned: &dyn Fn(Reg) -> bool,
    leaving: Option<&[Words]>,
) -> Vec<bool> {
    let fx: Vec<_> = ops.iter().map(|op| effect(op, places, lays)).collect();
    let top = (fx.iter().flatten())
        .flat_map(|(r, w)| r.iter().chain(w))
        .map(|&(a, n)| (a + n) as usize)
        .max()
        .unwrap_or(0);
    let words = top.div_ceil(64).max(1);
    let all = vec![u64::MAX; words];
    let mark = |v: &mut [u64], &(a, n): &Words, on: bool| {
        for r in a..(a + n).min(64 * words as u32) {
            let (w, bit) = (r as usize / 64, 1u64 << (r % 64));
            if on {
                v[w] |= bit;
            } else {
                v[w] &= !bit;
            }
        }
    };
    let exit = match leaving {
        Some(ws) => {
            let mut v = vec![0u64; words];
            ws.iter().for_each(|w| mark(&mut v, w, true));
            v
        }
        None => all.clone(),
    };
    let next = |i: usize| -> Vec<usize> {
        let mut v: Vec<usize> = (targets(&ops[i], tables).iter())
            .map(|&t| t as usize)
            .collect();
        if !matches!(ops[i], Op::Jump { .. } | Op::Switch { .. }) {
            v.push(i + 1);
        }
        v
    };
    let (mut live_in, mut live_out) = (
        vec![vec![0u64; words]; ops.len()],
        vec![all.clone(); ops.len()],
    );
    loop {
        let mut changed = false;
        for i in (0..ops.len()).rev() {
            let mut out = vec![0u64; words];
            for t in next(i) {
                let l = live_in.get(t).unwrap_or(&exit);
                out.iter_mut().zip(l).for_each(|(o, x)| *o |= x);
            }
            let inn = match &fx[i] {
                None if matches!(ops[i], Op::Leave(_) | Op::Fail { .. }) => exit.clone(),
                None => all.clone(),
                Some((r, w)) => {
                    let mut v = out.clone();
                    w.iter().for_each(|x| mark(&mut v, x, false));
                    r.iter().for_each(|x| mark(&mut v, x, true));
                    v
                }
            };
            live_out[i] = out;
            if inn != live_in[i] {
                live_in[i] = inn;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    (0..ops.len())
        .map(|i| {
            let Some((_, w)) = &fx[i] else { return true };
            let read = |r: Reg| pinned(r) || live_out[i][r as usize / 64] >> (r % 64) & 1 == 1;
            !pure(&ops[i]) || w.iter().any(|&(a, n)| (a..a + n).any(read))
        })
        .collect()
}

fn renumber(
    ops: &mut Vec<Op>,
    marks: &mut [(u32, u32)],
    tables: &mut [Vec<u32>],
    keep: &[bool],
    origins: &mut Vec<(u32, u32)>,
) {
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
    if !origins.is_empty() {
        let mut i = 0;
        origins.retain(|_| {
            i += 1;
            keep[i - 1]
        });
    }
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
    use cpl_next_ir::ir::Intr;
    use cpl_next_ir::ops;

    fn not(d: Reg, a: Reg) -> Op {
        let sig = ops::sig(Intr::Not, ops::Operand::Bool);
        Op::Unary {
            op: Intr::Not,
            sig,
            d,
            a,
        }
    }

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
            renumber(&mut ops, &mut marks, &mut [], &keep, &mut Vec::new());
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

    /// A move's readers read its source, a place indexed by it becomes one indexed by the
    /// source, and then the move goes. Leaving a region body may read any register, so there
    /// nothing goes; nor does anything in a register whose address is taken.
    #[test]
    fn a_register_nothing_reads_again_is_not_set() {
        let ops = vec![
            Op::Imm { d: 1, v: 5 },
            Op::Move { d: 2, s: 1, n: 1 },
            not(3, 2),
            Op::Move { d: 4, s: 3, n: 1 },
            Op::Peek { d: 6, p: 0 },
            Op::JumpIf { c: 6, t: 7 },
            Op::Imm { d: 7, v: 1 },
            Op::Leave(crate::compile::Leave::Return),
        ];
        let places = vec![PlaceCode {
            root: Root::Local(5),
            steps: vec![Step::Slot { i: 4, stride: 1 }],
        }];
        let lays = Layouts::default();
        let run = |pinned: &dyn Fn(Reg) -> bool, leaving| {
            let (mut ops, mut places) = (ops.clone(), places.clone());
            forward(&mut ops, &mut places, &lays, &[], pinned);
            let keep = unread(&ops, &places, &lays, &[], pinned, leaving);
            (
                format!("{:?} {:?}", ops[2], places.last().unwrap().steps),
                keep,
            )
        };
        let (not, keep) = run(&|_| false, Some(&[(0, 1)]));
        assert_eq!(
            not,
            format!("{:?} [Slot {{ i: 3, stride: 1 }}]", self::not(3, 1))
        );
        assert_eq!(keep, [true, false, true, false, true, true, false, true]);
        let (_, keep) = run(&|_| false, None);
        assert!(keep.iter().all(|&k| k));
        let (not, keep) = run(&|_| true, Some(&[(0, 1)]));
        assert_eq!(
            not,
            format!("{:?} [Slot {{ i: 4, stride: 1 }}]", self::not(3, 2))
        );
        assert!(keep.iter().all(|&k| k));
    }

    /// A place traps only where it indexes a cell, so storing to a global needs no origin.
    #[test]
    fn only_an_index_traps_in_a_place() {
        let places = [
            PlaceCode {
                root: Root::Global(0),
                steps: vec![Step::Off(1)],
            },
            PlaceCode {
                root: Root::Local(0),
                steps: vec![Step::Slot { i: 1, stride: 1 }],
            },
        ];
        assert!(!traps(&Op::Put { p: 0, s: 2, n: 1 }, &places));
        assert!(traps(&Op::Put { p: 1, s: 2, n: 1 }, &places));
        assert!(traps(&Op::Peek { d: 2, p: 1 }, &places));
    }

    /// Releasing or retaining a value runs no code that can trap, so a block whose ways leave
    /// different origins before its frame's releases still reads its origins from a table.
    #[test]
    fn a_release_needs_no_line() {
        assert!(!traps(&Op::Drop { s: 1, lay: 0 }, &[]));
        assert!(!traps(&Op::Retain { s: 1, lay: 0 }, &[]));
    }

    /// `t = x; t[i] = v; x = t` writes `x[i]` and then copies `x` to `t`, which nothing reads,
    /// so both copies go, also when a path sets `t` again. A jump past the copy back keeps both.
    #[test]
    fn a_copy_out_and_back_works_on_the_original() {
        let places = vec![PlaceCode {
            root: Root::Local(5),
            steps: vec![Step::Slot { i: 4, stride: 1 }],
        }];
        let lays = Layouts::default();
        let run = |ops: Vec<Op>| {
            let (mut ops, mut places) = (ops, places.clone());
            forward(&mut ops, &mut places, &lays, &[], &|_| false);
            let keep = unread(&ops, &places, &lays, &[], &|_| false, Some(&[(0, 1)]));
            let kept: Vec<String> = (ops.iter().zip(&keep))
                .filter(|(_, &k)| k)
                .map(|(op, _)| match place_of(&mut op.clone()) {
                    Some(&mut p) => format!("{op:?} {:?}", places[p as usize].root),
                    None => format!("{op:?}"),
                })
                .collect();
            kept
        };
        let put = Op::Put { p: 0, s: 3, n: 1 };
        let back = Op::Move { d: 2, s: 5, n: 1 };
        let leave = Op::Leave(crate::compile::Leave::Return);
        let kept = run(vec![Op::Move { d: 5, s: 2, n: 1 }, put, back, leave]);
        assert_eq!(kept, ["Put { p: 1, s: 3, n: 1 } Local(2)", "Leave(Return)"]);
        let jump = Op::JumpIf { c: 3, t: 4 };
        let kept = run(vec![Op::Move { d: 5, s: 2, n: 1 }, jump, put, back, leave]);
        assert_eq!(kept[2], "Put { p: 0, s: 3, n: 1 } Local(5)");
        let jump = Op::JumpIf { c: 3, t: 3 };
        let set = Op::Move { d: 5, s: 6, n: 1 };
        let kept = run(vec![
            Op::Move { d: 5, s: 2, n: 1 },
            jump,
            set,
            put,
            back,
            leave,
        ]);
        assert_eq!(
            kept,
            [
                "JumpIf { c: 3, t: 3 }",
                "Move { d: 2, s: 6, n: 1 }",
                "Put { p: 1, s: 3, n: 1 } Local(2)",
                "Leave(Return)"
            ]
        );
    }

    /// A move's readers past a join read its source when every jump into the join comes from
    /// after the move; a join a later jump also reaches keeps reading the move.
    #[test]
    fn a_move_reaches_past_a_join_only_its_own_jumps_reach() {
        let ops = |back: bool| {
            vec![
                Op::Move { d: 2, s: 1, n: 1 },
                Op::JumpUnless { c: 5, t: 3 },
                Op::Imm { d: 6, v: 1 },
                not(3, 2),
                Op::JumpIf { c: 5, t: 6 },
                Op::Imm { d: 7, v: 0 },
                not(4, 2),
                if back {
                    Op::Jump { t: 6 }
                } else {
                    Op::Imm { d: 8, v: 0 }
                },
                Op::Leave(crate::compile::Leave::Return),
            ]
        };
        let lays = Layouts::default();
        let run = |back| {
            let mut ops = ops(back);
            forward(&mut ops, &mut Vec::new(), &lays, &[], &|_| false);
            format!("{:?} {:?}", ops[3], ops[6])
        };
        assert_eq!(run(false), format!("{:?} {:?}", not(3, 1), not(4, 1)));
        assert_eq!(run(true), format!("{:?} {:?}", not(3, 1), not(4, 2)));
    }

    /// A drop of a register still zero on every way in becomes a zero, and both go when
    /// nothing reads the register; a drop after the register is set again stays.
    #[test]
    fn a_release_of_a_register_still_zero_goes() {
        let mut ops = vec![
            Op::Zero { d: 1, n: 1 },
            Op::JumpUnless { c: 5, t: 3 },
            Op::Imm { d: 6, v: 1 },
            Op::Drop { s: 1, lay: 0 },
            Op::Imm { d: 1, v: 9 },
            Op::Drop { s: 1, lay: 0 },
            Op::Leave(crate::compile::Leave::Return),
        ];
        let mut lays = Layouts::default();
        lays.size.push(1);
        lays.flat.push(Some(vec![0]));
        forward(&mut ops, &mut Vec::new(), &lays, &[], &|_| false);
        assert_eq!(
            format!("{:?} {:?}", ops[3], ops[5]),
            "Zero { d: 1, n: 1 } Drop { s: 1, lay: 0 }"
        );
        let keep = unread(&ops, &[], &lays, &[], &|_| false, Some(&[(0, 1)]));
        assert!(!keep[0] && !keep[3] && keep[5]);
    }

    /// A store through a place rooted in a register still zero sets it, so its drop stays.
    #[test]
    fn unzero_stops_at_a_store_through_the_zeroed_register() {
        let mut ops = vec![
            Op::Zero { d: 1, n: 1 },
            Op::Put { p: 0, s: 2, n: 1 },
            Op::Drop { s: 1, lay: 0 },
            Op::Leave(crate::compile::Leave::Return),
        ];
        let mut places = vec![PlaceCode {
            root: Root::Local(1),
            steps: vec![],
        }];
        let mut lays = Layouts::default();
        lays.size.push(1);
        lays.flat.push(Some(vec![0]));
        forward(&mut ops, &mut places, &lays, &[], &|_| false);
        assert_eq!(format!("{:?}", ops[2]), "Drop { s: 1, lay: 0 }");
    }

    /// Leaving a fn reads its result, its lent parameters and the locals it drops, not
    /// every local: a local's last write that nothing else reads goes.
    #[test]
    fn leaving_a_fn_reads_only_what_its_caller_sees() {
        let ops = [
            Op::Imm { d: 3, v: 1 },
            Op::Imm { d: 0, v: 2 },
            Op::Leave(crate::compile::Leave::Return),
        ];
        let lays = Layouts::default();
        let kept = |leaving: &[Words]| unread(&ops, &[], &lays, &[], &|_| false, Some(leaving));
        assert_eq!(kept(&[(0, 1)]), [false, true, true]);
        assert_eq!(kept(&[(0, 1), (3, 1)]), [true, true, true]);
    }

    /// A kept origin around moves goes; around a `Take`, which can trap, it stays.
    #[test]
    fn a_line_nothing_can_trap_at_is_not_kept() {
        let ops = vec![
            Op::Enter { origin: 3, save: 9 },
            Op::Drop { s: 1, lay: 0 },
            Op::Enter { origin: 0, save: 8 },
            Op::Move { d: 1, s: 2, n: 1 },
            Op::Restore { save: 8 },
            Op::Take { d: 1, p: 0, n: 1 },
            Op::Restore { save: 9 },
            Op::Enter { origin: 0, save: 7 },
            Op::Move { d: 1, s: 2, n: 1 },
            Op::Restore { save: 7 },
        ];
        let keep = quiet(&ops, &[], &[]);
        assert_eq!(
            keep,
            [true, true, false, true, false, true, true, false, true, false]
        );
    }

    /// Around a `Take`, which can trap, a pair entering no origin goes, and so does one entering
    /// the origin already set; one entering another origin stays, as does one whose set origin is
    /// reached by a jump that may set another. A jump inside a pair keeps it from going only
    /// when it comes from outside. A pair whose save a call stack reads stays.
    #[test]
    fn a_line_already_set_is_not_entered_again() {
        let ops = vec![
            Op::Enter { origin: 0, save: 9 },
            Op::Take { d: 1, p: 0, n: 1 },
            Op::Restore { save: 9 },
            Op::At { origin: 5 },
            Op::Enter { origin: 5, save: 9 },
            Op::Take { d: 1, p: 0, n: 1 },
            Op::Restore { save: 9 },
            Op::Enter { origin: 4, save: 9 },
            Op::Take { d: 1, p: 0, n: 1 },
            Op::Restore { save: 9 },
            Op::Imm { d: 1, v: 0 },
            Op::Enter { origin: 4, save: 9 },
            Op::Take { d: 1, p: 0, n: 1 },
            Op::Restore { save: 9 },
            Op::JumpIf { c: 1, t: 10 },
            Op::Enter { origin: 0, save: 9 },
            Op::JumpIf { c: 1, t: 18 },
            Op::Take { d: 1, p: 0, n: 1 },
            Op::Restore { save: 9 },
            Op::Enter { origin: 0, save: 9 },
            Op::Take { d: 1, p: 0, n: 1 },
            Op::Restore { save: 9 },
            Op::JumpIf { c: 1, t: 21 },
        ];
        let keep = quiet(&ops, &[], &[]);
        let kept: Vec<usize> = (0..ops.len()).filter(|&i| keep[i]).collect();
        let chained = quiet(&ops, &[], &[9]);
        assert_eq!(chained.iter().filter(|&&k| k).count(), ops.len());
        assert_eq!(
            kept,
            [1, 3, 5, 7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 20, 21, 22]
        );
    }
}
