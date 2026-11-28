//! Runs engine code. Registers are words of one stack; storage is counted heap objects,
//! copied when written while shared. Flows, origins and events follow the reference
//! interpreter's rules step for step.
use crate::compile::*;
use crate::layout::Lay;
use crate::Stop;
use cpl_next_ir::ops::{self, Fault};
use cpl_next_ir::run::{Io, Limits, RunInput, Stack};
use cpl_next_ir::run::{Outcome, Trap, Written};

struct Obj {
    rc: u32,
    elem: Lay,
    len: usize,
    words: Vec<u64>,
}

enum Flow {
    Normal,
    Break,
    Continue,
    /// The value is in the result words of the frame the `return` ran in.
    Return,
    /// The caller's region body left early, so this generator stops here.
    Exit(Box<Flow>),
}

#[derive(Clone, Copy)]
struct RegionAt {
    fr: usize,
    body: u32,
    var: Reg,
}
struct Frame {
    fp: usize,
    /// Its fn and the frame that called it: a trap's call stack.
    f: u32,
    parent: Option<usize>,
    end: usize,
    region: Option<RegionAt>,
    pending: Option<Flow>,
}

/// Where a word is: `i` into the stack when `h` is 0, else into storage `h`'s words. Handle 0
/// is never storage.
#[derive(Clone, Copy)]
struct Loc {
    h: u32,
    i: usize,
}
impl Loc {
    fn stack(i: usize) -> Loc {
        Loc { h: 0, i }
    }
    fn heap(h: u32, i: usize) -> Loc {
        Loc { h, i }
    }
    fn off(self, n: usize) -> Loc {
        Loc {
            i: self.i + n,
            ..self
        }
    }
}

struct M<'a> {
    /// The frame whose code is running: a region body runs in its caller's.
    cur: usize,
    /// Each block run a trap left, innermost first: the block, its op, the frame and `at`.
    trail: Vec<(u32, usize, usize, u32)>,
    c: &'a Code,
    stack: Vec<u64>,
    /// Object 0 is no object: a handle of 0 has none yet.
    heap: Vec<Obj>,
    free: Vec<u32>,
    frames: Vec<Frame>,
    out: Written,
    input: ops::Input,
    rand: u64,
    at: u32,
    texts: Vec<u32>,
    scratch: Vec<u32>,
    /// Steps left: each fn entered and each jump taken back is one.
    steps: u64,
    room: Stack,
    /// `Limits::memory`, which each storage is made under.
    memory: i64,
    /// No input: a read stops the run, its value being decided at run time.
    closed: bool,
    /// Whether a run with no input tried to read.
    read: bool,
}
type R<T> = Result<T, Trap>;

fn broken<T>(rule: &'static str) -> R<T> {
    Err(Fault::Internal(rule).into())
}

fn start(c: &Code, input: Option<RunInput>, steps: u64) -> M<'_> {
    let closed = input.is_none();
    let (io, limits, out) = input.map_or((Io::Fixed(Vec::new()), Limits::default(), None), |i| {
        (i.io, i.limits, i.out)
    });
    M {
        c,
        cur: 0,
        trail: Vec::new(),
        stack: vec![0; c.globals as usize],
        heap: vec![Obj {
            rc: 0,
            elem: 0,
            len: 0,
            words: Vec::new(),
        }],
        free: Vec::new(),
        frames: Vec::new(),
        out: Written::to(out),
        input: io.input(),
        rand: ops::RAND_SEED,
        at: 0,
        texts: vec![0; c.texts.len()],
        scratch: Vec::new(),
        steps,
        room: (c.stack).with(limits),
        memory: i64::try_from(limits.memory).unwrap_or(i64::MAX),
        closed,
        read: false,
    }
}

pub(crate) fn run(c: &Code, input: RunInput) -> Outcome {
    let (m, trap) = ran(c, input);
    m.outcome(trap)
}

/// Runs the program, then releases its globals and its kept text constants if it ended: what
/// it did, and how many storage objects are still held. Ownership releases every value it makes, so that is 0.
pub(crate) fn run_held(c: &Code, input: RunInput) -> (Outcome, usize) {
    let (mut m, trap) = ran(c, input);
    if trap.is_none() {
        for (&at, &lay) in c.global_at.iter().zip(&c.global_lays) {
            m.release(lay, Loc::stack(at as usize));
        }
        let texts: Vec<u32> = m.texts.iter().copied().filter(|&h| h != 0).collect();
        m.dec(texts);
    }
    let held = m.heap.len() - 1 - m.free.len();
    (m.outcome(trap), held)
}

fn ran(c: &Code, input: RunInput) -> (M<'_>, Option<Trap>) {
    let mut m = start(c, Some(input), u64::MAX);
    let fp = c.globals as usize;
    let trap = m.run_fn(c.main, fp, None).err().map(|t| Trap {
        origin: m.trail_origins().first().copied().unwrap_or(0),
        calls: m.call_stack(),
        ..t
    });
    (m, trap)
}

/// Runs the program with no input for at most `steps` steps: the words global `g` holds when
/// it ends. A read of input stops it.
pub(crate) fn eval(c: &Code, gs: &[usize], steps: u64) -> Result<Vec<Vec<u64>>, Stop> {
    let mut m = start(c, None, steps);
    match m.run_fn(c.main, c.globals as usize, None) {
        Ok(_) => Ok((gs.iter())
            .map(|&g| {
                let at = c.global_at[g] as usize;
                let end = (c.global_at.get(g + 1)).map_or(c.globals as usize, |&e| e as usize);
                m.stack[at..end].to_vec()
            })
            .collect()),
        Err(_) if m.read => Err(Stop::Input),
        Err(_) if m.steps == 0 => Err(Stop::Budget),
        Err(t) => Err(Stop::Trap(Trap {
            origin: m.trail_origins().first().copied().unwrap_or(0),
            calls: m.call_stack(),
            ..t
        })),
    }
}

impl M<'_> {
    fn outcome(self, trap: Option<Trap>) -> Outcome {
        self.out.end(trap)
    }
    /// One step of the budget.
    fn step(&mut self) -> R<()> {
        if self.steps == 0 {
            return broken("the step budget ran out");
        }
        self.steps -= 1;
        Ok(())
    }
    fn alloc(&mut self, elem: Lay, len: usize, words: Vec<u64>) -> u32 {
        let o = Obj {
            rc: 1,
            elem,
            len,
            words,
        };
        match self.free.pop() {
            Some(h) => {
                self.heap[h as usize] = o;
                h
            }
            None => {
                self.heap.push(o);
                (self.heap.len() - 1) as u32
            }
        }
    }
    /// Counts one more holder of each handle in the value at stack address `a`.
    fn retain(&mut self, lay: Lay, a: usize) {
        let c = self.c;
        if let Some(at) = &c.lays.flat[lay as usize] {
            for &o in at {
                let h = self.stack[a + o as usize];
                if h != 0 {
                    self.heap[h as usize].rc += 1;
                }
            }
            return;
        }
        let mut hs = std::mem::take(&mut self.scratch);
        self.c.lays.handles(lay, &self.stack[a..], &mut hs);
        for h in hs.drain(..) {
            self.heap[h as usize].rc += 1;
        }
        self.scratch = hs;
    }
    fn retain_heap(&mut self, lay: Lay, h: u32, i: usize, times: usize) {
        if times == 0 {
            return;
        }
        let mut hs = std::mem::take(&mut self.scratch);
        self.c
            .lays
            .handles(lay, &self.heap[h as usize].words[i..], &mut hs);
        for x in hs.drain(..) {
            self.heap[x as usize].rc += times as u32;
        }
        self.scratch = hs;
    }
    /// Drops the value at `loc`: what it holds loses a holder, and an object with none is freed.
    fn release(&mut self, lay: Lay, loc: Loc) {
        let c = self.c;
        if let Some(at) = &c.lays.flat[lay as usize] {
            for &o in at {
                let h = self.word(loc.off(o as usize));
                if h == 0 {
                    continue;
                }
                let obj = &mut self.heap[h as usize];
                if obj.rc > 1 {
                    obj.rc -= 1;
                } else {
                    self.dec(vec![h as u32]);
                }
            }
            return;
        }
        let mut hs = Vec::new();
        let words = match loc.h {
            0 => &self.stack[loc.i..],
            h => &self.heap[h as usize].words[loc.i..],
        };
        c.lays.handles(lay, words, &mut hs);
        self.dec(hs);
    }
    fn dec(&mut self, mut hs: Vec<u32>) {
        while let Some(h) = hs.pop() {
            let o = &mut self.heap[h as usize];
            o.rc -= 1;
            if o.rc > 0 {
                continue;
            }
            let words = std::mem::take(&mut o.words);
            let (elem, len) = (o.elem, o.len);
            let stride = self.c.lays.size[elem as usize] as usize;
            match &self.c.lays.flat[elem as usize] {
                Some(at) => {
                    for &o in at {
                        let hs_at = (0..len).map(|i| words[i * stride + o as usize]);
                        hs.extend(hs_at.filter(|&x| x != 0).map(|x| x as u32));
                    }
                }
                None => {
                    for i in 0..len {
                        self.c.lays.handles(elem, &words[i * stride..], &mut hs);
                    }
                }
            }
            self.free.push(h);
        }
    }
    fn word(&self, loc: Loc) -> u64 {
        match loc.h {
            0 => self.stack[loc.i],
            h => self.heap[h as usize].words[loc.i],
        }
    }
    fn set_word(&mut self, loc: Loc, v: u64) {
        match loc.h {
            0 => self.stack[loc.i] = v,
            h => self.heap[h as usize].words[loc.i] = v,
        }
    }
    /// Copies `n` stack words at `s` to `d`.
    fn copy(&mut self, s: usize, d: usize, n: usize) {
        if n == 1 {
            self.stack[d] = self.stack[s];
        } else {
            self.stack.copy_within(s..s + n, d);
        }
    }
    /// Copies `n` words at `loc` to stack address `a`.
    fn read(&mut self, loc: Loc, a: usize, n: usize) {
        match loc.h {
            0 => self.copy(loc.i, a, n),
            h => {
                self.stack[a..a + n].copy_from_slice(&self.heap[h as usize].words[loc.i..loc.i + n])
            }
        }
    }
    /// Copies `n` words at stack address `a` to `loc`.
    fn write(&mut self, loc: Loc, a: usize, n: usize) {
        match loc.h {
            0 => self.copy(a, loc.i, n),
            h => {
                self.heap[h as usize].words[loc.i..loc.i + n].copy_from_slice(&self.stack[a..a + n])
            }
        }
    }
    /// The storage whose handle is at `loc`, made this holder's own first.
    fn unique(&mut self, loc: Loc) -> R<u32> {
        let h = self.word(loc) as u32;
        if h == 0 {
            return broken("a cell of a value that is not storage");
        }
        if self.heap[h as usize].rc == 1 {
            return Ok(h);
        }
        let o = &self.heap[h as usize];
        let (elem, len, words) = (o.elem, o.len, o.words.clone());
        self.heap[h as usize].rc -= 1;
        let n = self.alloc(elem, len, words);
        let stride = self.c.lays.size[elem as usize] as usize;
        for i in 0..len {
            self.retain_heap(elem, n, i * stride, 1);
        }
        self.set_word(loc, u64::from(n));
        Ok(n)
    }
    /// Where place `p` is, each storage on the way made this holder's own: it will be written.
    #[inline(always)]
    fn loc(&mut self, fp: usize, p: u32) -> R<Loc> {
        self.place(fp, p, true)
    }
    /// Where place `p` is, to be read: storage shared with another holder stays shared.
    #[inline(always)]
    fn look(&mut self, fp: usize, p: u32) -> R<Loc> {
        self.place(fp, p, false)
    }
    #[inline(always)]
    fn place(&mut self, fp: usize, p: u32, own: bool) -> R<Loc> {
        let pc = &self.c.places[p as usize];
        let mut loc = match pc.root {
            Root::Local(r) => Loc::stack(fp + r as usize),
            Root::Global(g) => Loc::stack(g as usize),
        };
        for st in &pc.steps {
            loc = match *st {
                Step::Off(at) => loc.off(at as usize),
                Step::Deref => Loc::stack(self.word(loc) as usize),
                Step::Slot { i, stride } => {
                    let idx = self.stack[fp + i as usize] as i64;
                    let h = match own {
                        true => self.unique(loc)?,
                        false => match self.word(loc) as u32 {
                            0 => return broken("a cell of a value that is not storage"),
                            h => h,
                        },
                    };
                    match usize::try_from(idx) {
                        Ok(idx) if idx < self.heap[h as usize].len => {
                            Loc::heap(h, idx * stride as usize)
                        }
                        _ => return broken("a cell outside its storage"),
                    }
                }
            };
        }
        Ok(loc)
    }
    fn size(&self, lay: Lay) -> usize {
        self.c.lays.size[lay as usize] as usize
    }

    /// Runs fn `f` in a frame at `fp` whose parameters are already in place. Returns how a
    /// region body left, if it did.
    fn run_fn(&mut self, f: u32, fp: usize, region: Option<RegionAt>) -> R<Option<Flow>> {
        self.step()?;
        let fc = &self.c.funcs[f as usize];
        // Spec 9.6: no depth counting in the language; the stack runs out at the run's limit.
        let end = fp + fc.frame as usize;
        self.room.enter(f as usize, self.frames.len())?;
        if self.stack.len() < end {
            self.stack.resize(end.max(self.stack.len() * 2), 0);
        }
        let parent = (!self.frames.is_empty()).then_some(self.cur);
        self.frames.push(Frame {
            fp,
            f,
            parent,
            end,
            region,
            pending: None,
        });
        let fr = self.frames.len() - 1;
        self.cur = fr;
        let flow = self.exec(fc.block, fr)?;
        let Some(frame) = self.frames.pop() else {
            return broken("a call returned with no frame");
        };
        self.cur = frame.parent.unwrap_or(0);
        for &(at, lay) in &fc.drops {
            self.release(lay, Loc::stack(fp + at as usize));
        }
        Ok(match flow {
            Flow::Exit(f) => Some(*f),
            _ => frame.pending,
        })
    }
    /// A call from frame `fr`: arguments to the new frame, the run, lent values back.
    fn call(&mut self, c: u32, fr: usize, region: Option<RegionAt>) -> R<Option<Flow>> {
        let cc = &self.c.calls[c as usize];
        let fc = &self.c.funcs[cc.f as usize];
        let fp = self.frames[fr].fp;
        let Some(top) = self.frames.last().map(|f| f.end) else {
            return broken("a call with no frame");
        };
        let end = top + fc.frame as usize;
        if self.stack.len() < end {
            self.stack.resize(end.max(self.stack.len() * 2), 0);
        }
        for &(at, lay) in &fc.drops {
            let a = top + at as usize;
            let n = self.size(lay);
            self.stack[a..a + n].fill(0);
        }
        for (a, &(at, n)) in cc.args.iter().zip(&fc.params) {
            let to = top + at as usize;
            match *a {
                ArgCode::Move { s, n } => {
                    let s = fp + s as usize;
                    self.stack.copy_within(s..s + n as usize, to);
                }
                ArgCode::Lend { p } => {
                    let loc = self.loc(fp, p)?;
                    self.read(loc, to, n as usize);
                }
            }
        }
        let exit = self.run_fn(cc.f, top, region)?;
        let lent = fc.params.iter().zip(&fc.lent).filter(|(_, l)| **l);
        let lends = cc.args.iter().filter_map(|a| match a {
            ArgCode::Lend { p } => Some(*p),
            ArgCode::Move { .. } => None,
        });
        for (p, (&(at, n), _)) in lends.zip(lent) {
            let loc = self.loc(fp, p)?;
            self.write(loc, top + at as usize, n as usize);
        }
        if region.is_none() && fc.ret > 0 {
            let d = fp + cc.d as usize;
            self.stack.copy_within(top..top + fc.ret as usize, d);
        }
        Ok(exit)
    }
    /// The calls a trap left, innermost first: each frame's fn, and each call inlined into
    /// it, from the chain of the op it stopped at, with the origin running in it.
    fn call_stack(&self) -> Vec<(usize, u32)> {
        let origins = self.trail_origins();
        let mut calls = Vec::new();
        let (mut k, mut pos) = (Some(self.cur), 0);
        while let Some(fr) = k.and_then(|i| self.frames.get(i)) {
            let mut origin = 0;
            if let Some(i) = self.trail[pos..].iter().position(|t| Some(t.2) == k) {
                let (b, pc, _, _) = self.trail[pos + i];
                let outside = origins.get(pos + i + 1).copied().unwrap_or(0);
                origin = origins[pos + i];
                pos += i + 1;
                let marks = &self.c.marks[b as usize];
                let at = marks
                    .partition_point(|m| m.0 as usize <= pc)
                    .saturating_sub(1);
                let chain = marks
                    .get(at)
                    .map_or(&[][..], |m| &self.c.chains[m.1 as usize]);
                let saved = self.c.origins[b as usize]
                    .get(pc)
                    .map(|&(_, s)| &self.c.saved[s as usize]);
                for (j, &(g, save)) in chain.iter().enumerate().rev() {
                    calls.push((g as usize, origin));
                    origin = match saved.map(|s| s[j]) {
                        Some(SAVED) | None => self.stack[fr.fp + save as usize] as u32,
                        Some(l) => l,
                    };
                    if origin == INHERIT {
                        origin = outside;
                    }
                }
            }
            calls.push((fr.f as usize, origin));
            k = fr.parent;
        }
        calls
    }
    /// The origin each block run of the trail stopped at, innermost first: from the block's
    /// table, or `at` for a block with none. A block that set none has the origin outside it.
    fn trail_origins(&self) -> Vec<u32> {
        let mut origins = vec![0; self.trail.len()];
        let mut outside = 0;
        for (i, &(b, pc, _, at)) in self.trail.iter().enumerate().rev() {
            let own = self.c.origins[b as usize].get(pc).map_or(at, |&(l, _)| l);
            if own != INHERIT {
                outside = own;
            }
            origins[i] = outside;
        }
        origins
    }
    fn enter(&mut self, origin: u32) -> u32 {
        if origin != 0 {
            self.at = origin;
        }
        self.at
    }

    fn exec(&mut self, b: u32, fr: usize) -> R<Flow> {
        let mut pc = 0;
        let outside = std::mem::replace(&mut self.at, INHERIT);
        let flow = self.run_block(b, fr, &mut pc);
        if flow.is_err() {
            self.trail.push((b, pc.saturating_sub(1), fr, self.at));
        }
        self.at = outside;
        flow
    }
    #[inline(always)]
    fn run_block(&mut self, b: u32, fr: usize, pc: &mut usize) -> R<Flow> {
        let c = self.c;
        let ops = &c.blocks[b as usize];
        let fp = self.frames[fr].fp;
        while let Some(&op) = ops.get(*pc) {
            *pc += 1;
            let r = |x: Reg| fp + x as usize;
            match op {
                Op::Imm { d, v } => self.stack[r(d)] = v,
                Op::Addr { d, s } => self.stack[r(d)] = r(s) as u64,
                Op::Move { d, s, n } => self.copy(r(s), r(d), n as usize),
                Op::Load { d, p, lay } => {
                    let loc = self.look(fp, p)?;
                    self.read(loc, r(d), self.size(lay));
                }
                Op::Peek { d, p } => {
                    let loc = self.look(fp, p)?;
                    self.stack[r(d)] = self.word(loc);
                }
                Op::Drop { s, lay } => self.release(lay, Loc::stack(r(s))),
                Op::Text { d, t } => {
                    let mut h = self.texts[t as usize];
                    if h == 0 {
                        let s = &c.texts[t as usize];
                        h = self.alloc(c.char_lay, s.len(), s.bytes().map(u64::from).collect());
                        self.texts[t as usize] = h;
                    }
                    self.heap[h as usize].rc += 1;
                    self.stack[r(d)] = u64::from(h);
                }
                Op::Int { op, d, a, b } => {
                    let v = ops::int(op, self.stack[r(a)] as i64, self.stack[r(b)] as i64)?;
                    self.stack[r(d)] = v as u64;
                }
                Op::Word { op, d, a, b } => {
                    self.stack[r(d)] = ops::word(op, self.stack[r(a)], self.stack[r(b)])?;
                }
                Op::Wide {
                    op,
                    d,
                    a,
                    b,
                    narrow,
                } => {
                    let y = if narrow {
                        i128::from(self.stack[r(b)] as i64)
                    } else {
                        wide(&self.stack, r(b))
                    };
                    let v = ops::wide(op, wide(&self.stack, r(a)), y)?;
                    set_wide(&mut self.stack, r(d), v);
                }
                Op::WideCmp { op, d, a, b } => {
                    let (x, y) = (wide(&self.stack, r(a)), wide(&self.stack, r(b)));
                    self.stack[r(d)] = u64::from(ops::compare(op, x, y)?);
                }
                Op::Unary { op, sig, d, a } => {
                    let hi = if sig.args[0].words() == 2 {
                        self.stack[r(a) + 1]
                    } else {
                        0
                    };
                    let v = ops::unary(op, sig, [self.stack[r(a)], hi])?;
                    let n = sig.res.words();
                    self.stack[r(d)..r(d) + n].copy_from_slice(&v[..n]);
                }
                Op::Float { op, d, a, b } => {
                    let (x, y) = (
                        f64::from_bits(self.stack[r(a)]),
                        f64::from_bits(self.stack[r(b)]),
                    );
                    self.stack[r(d)] = ops::float(op, x, y)?.to_bits();
                }
                Op::Cmp {
                    op,
                    d,
                    a,
                    b,
                    kind: Kind::F64,
                } => {
                    let (x, y) = (
                        f64::from_bits(self.stack[r(a)]),
                        f64::from_bits(self.stack[r(b)]),
                    );
                    self.stack[r(d)] = u64::from(ops::compare_float(op, x, y)?);
                }
                Op::Cmp { op, d, a, b, kind } => {
                    let (x, y) = (self.stack[r(a)], self.stack[r(b)]);
                    let (x, y) = match kind {
                        Kind::Int => (i128::from(x as i64), i128::from(y as i64)),
                        _ => (i128::from(x), i128::from(y)),
                    };
                    self.stack[r(d)] = u64::from(ops::compare(op, x, y)?);
                }
                Op::At { origin } => self.at = origin,
                Op::Enter { origin, save } => self.stack[r(save)] = u64::from(self.enter(origin)),
                Op::Call { c: k } => {
                    self.call(k, fr, None)?;
                }
                Op::Region {
                    c: k,
                    body,
                    var,
                    lend,
                    jumps,
                } => {
                    self.enter(c.calls[k as usize].origin);
                    let exit = self.call(k, fr, Some(RegionAt { fr, body, var }))?;
                    let to = match exit {
                        None => continue,
                        // A generator's region is a loop body: `break` ends the loop.
                        Some(Flow::Break) if !lend => continue,
                        Some(Flow::Break) => jumps.brk,
                        Some(Flow::Continue) => jumps.cont,
                        Some(Flow::Normal) => continue,
                        Some(Flow::Return | Flow::Exit(_)) => jumps.other,
                    };
                    if to == LEAVE {
                        return Ok(exit.unwrap_or(Flow::Normal));
                    }
                    *pc = to as usize;
                }
                Op::Yield { s, lay } => {
                    let Some(rg) = self.frames[fr].region else {
                        return broken("yield with no caller code");
                    };
                    let v = self.frames[rg.fr].fp + rg.var as usize;
                    self.release(lay, Loc::stack(v));
                    let n = self.size(lay);
                    self.stack.copy_within(r(s)..r(s) + n, v);
                    let was = self.cur;
                    self.cur = rg.fr;
                    let flow = self.exec(rg.body, rg.fr)?;
                    self.cur = was;
                    match flow {
                        Flow::Normal | Flow::Continue => {}
                        other => return Ok(Flow::Exit(Box::new(other))),
                    }
                }
                Op::LendOut { p, lay } => {
                    let Some(rg) = self.frames[fr].region else {
                        return broken("lend with no caller code");
                    };
                    let (v, n) = (self.frames[rg.fr].fp + rg.var as usize, self.size(lay));
                    let loc = self.loc(fp, p)?;
                    self.read(loc, v, n);
                    let was = self.cur;
                    self.cur = rg.fr;
                    let flow = self.exec(rg.body, rg.fr)?;
                    self.cur = was;
                    let loc = self.loc(fp, p)?;
                    self.write(loc, v, n);
                    self.stack[v..v + n].fill(0);
                    // The rest of the lender still runs; the caller leaves after it.
                    if !matches!(flow, Flow::Normal) {
                        self.frames[fr].pending = Some(flow);
                    }
                }
                Op::Jump { t } => {
                    if (t as usize) < *pc {
                        self.step()?;
                    }
                    *pc = t as usize
                }
                Op::JumpUnless { c: x, t } => {
                    if self.stack[r(x)] == 0 {
                        if (t as usize) < *pc {
                            self.step()?;
                        }
                        *pc = t as usize;
                    }
                }
                Op::JumpIf { c: x, t } => {
                    if self.stack[r(x)] != 0 {
                        if (t as usize) < *pc {
                            self.step()?;
                        }
                        *pc = t as usize;
                    }
                }
                Op::Switch { c: x, table } => {
                    match c.tables[table as usize].get(self.stack[r(x)] as usize) {
                        Some(&t) => *pc = t as usize,
                        None => return broken("a switch on a tag past its arms"),
                    }
                }
                Op::Leave(how) => {
                    return Ok(match how {
                        Leave::Break => Flow::Break,
                        Leave::Continue => Flow::Continue,
                        Leave::Return => Flow::Return,
                    })
                }
                Op::Check {
                    c: x,
                    kind,
                    origin,
                    msg,
                } => {
                    if self.stack[r(x)] == 0 {
                        self.enter(origin);
                        return Err(Trap {
                            msg: c.msgs[msg as usize].clone(),
                            ..Fault::Check(kind).into()
                        });
                    }
                }
                Op::Guard { op, c: x } => ops::check(op, self.stack[r(x)] != 0)?,
                Op::Out { r: s, n, owned } => {
                    let h = self.stack[r(s)] as u32;
                    if h == 0 {
                        return broken("output of non-storage");
                    }
                    let stride = self.size(self.heap[h as usize].elem).max(1);
                    let o = &self.heap[h as usize];
                    let n = ops::out_len(self.stack[r(n)] as i64, o.len);
                    for &w in o.words.iter().step_by(stride).take(n) {
                        self.out.text.push(char::from(byte(w)?));
                    }
                    if owned {
                        self.dec(vec![h]);
                    }
                }
                Op::Showing { d } => self.stack[r(d)] = u64::from(self.out.showing()),
                Op::Capture => self.out.mark(),
                Op::Flush => self.out.flush(),
                Op::Event { e } => {
                    let ev = &c.events[e as usize];
                    (self.out).fire(ev.kind.clone(), ev.func, ev.site, ev.vals);
                }
                Op::RawEmpty { d, elem } => {
                    self.stack[r(d)] = u64::from(self.alloc(elem, 0, Vec::new()))
                }
                Op::RawNew { d, n, fill, elem } => {
                    let cell = self.c.lays.cell_bytes(elem);
                    let len = ops::storage_size(self.stack[r(n)] as i64, cell, self.memory)?;
                    let stride = self.size(elem);
                    let cell = &self.stack[r(fill)..r(fill) + stride];
                    let words = if stride == 1 {
                        vec![cell[0]; len]
                    } else {
                        cell.repeat(len)
                    };
                    let mut hs = Vec::new();
                    c.lays.handles(elem, cell, &mut hs);
                    for h in &hs {
                        self.heap[*h as usize].rc += len as u32;
                    }
                    self.dec(hs);
                    self.stack[r(d)] = u64::from(self.alloc(elem, len, words));
                }
                Op::RawGet {
                    d,
                    r: s,
                    i,
                    elem,
                    owned,
                } => {
                    let h = self.stack[r(s)] as u32;
                    if h == 0 {
                        return broken("read of non-storage");
                    }
                    let len = self.heap[h as usize].len;
                    let at = ops::cell(self.stack[r(i)] as i64, len, false)? * self.size(elem);
                    self.read(Loc::heap(h, at), r(d), self.size(elem));
                    self.retain(elem, r(d));
                    if owned {
                        self.dec(vec![h]);
                    }
                }
                Op::RawSet { p, i, x, elem } => {
                    let loc = self.loc(fp, p)?;
                    let h = self.word(loc) as u32;
                    if h == 0 {
                        return broken("write to non-storage");
                    }
                    let len = self.heap[h as usize].len;
                    let at = ops::cell(self.stack[r(i)] as i64, len, true)? * self.size(elem);
                    let h = self.unique(loc)?;
                    self.release(elem, Loc::heap(h, at));
                    self.write(Loc::heap(h, at), r(x), self.size(elem));
                }
                Op::RawTake { d, p, i, elem } => {
                    let loc = self.loc(fp, p)?;
                    if self.word(loc) == 0 {
                        return broken("move from non-storage");
                    }
                    let len = self.heap[self.word(loc) as usize].len;
                    let size = self.size(elem);
                    let at = ops::cell(self.stack[r(i)] as i64, len, false)? * size;
                    let h = self.unique(loc)?;
                    self.read(Loc::heap(h, at), r(d), size);
                    self.heap[h as usize].words[at..at + size].fill(0);
                }
                Op::RawCap { d, n, elem } => {
                    let cell = self.c.lays.cell_bytes(elem);
                    let len = ops::storage_size(self.stack[r(n)] as i64, cell, self.memory)?;
                    let words = vec![0; len * self.size(elem)];
                    self.stack[r(d)] = u64::from(self.alloc(elem, len, words));
                }
                Op::InputLeft { .. } | Op::ReadInt { .. } | Op::ReadByte { .. } if self.closed => {
                    self.read = true;
                    return broken("a read with no input");
                }
                Op::InputLeft { d } => {
                    self.stack[r(d)] = u64::from(ops::Input::left(&mut self.input, &mut self.out))
                }
                Op::ReadByte { d } => {
                    self.stack[r(d)] = ops::Input::byte(&mut self.input, &mut self.out) as u64
                }
                Op::PeekByte { d, k } => {
                    self.stack[r(d)] =
                        ops::Input::peek(&mut self.input, &mut self.out, self.stack[r(k)] as i64)
                            as u64;
                }
                Op::RandWord { d } => self.stack[r(d)] = ops::rand_word(&mut self.rand),
                Op::ReadInt { d } => {
                    let t = ops::Input::token(&mut self.input, &mut self.out);
                    let v =
                        ops::read_int(t.as_deref()).map_err(|f| Trap::at_token(f, t.as_deref()))?;
                    self.stack[r(d)] = v as u64;
                }
                Op::Fail { rule } => return broken(rule),
                Op::GetG { d, a, n } => {
                    self.copy(a as usize, r(d), n as usize);
                }
                Op::PutG { a, s, n } => {
                    self.copy(r(s), a as usize, n as usize);
                }
                Op::Retain { s, lay } => self.retain(lay, r(s)),
                Op::Zero { d, n } => self.stack[r(d)..r(d) + n as usize].fill(0),
                Op::Restore { save } => self.at = self.stack[r(save)] as u32,
                Op::Take { d, p, n } => {
                    let loc = self.loc(fp, p)?;
                    self.read(loc, r(d), n as usize);
                }
                Op::Put { p, s, n } => {
                    let loc = self.loc(fp, p)?;
                    self.write(loc, r(s), n as usize);
                }
            }
        }
        Ok(Flow::Normal)
    }
}

/// A `Char`'s word as its byte.
/// The `I128` at `r`: two words, low first.
fn wide(s: &[u64], r: usize) -> i128 {
    (u128::from(s[r]) | (u128::from(s[r + 1]) << 64)) as i128
}
fn set_wide(s: &mut [u64], r: usize, v: i128) {
    (s[r], s[r + 1]) = (v as u64, ((v as u128) >> 64) as u64);
}

fn byte(w: u64) -> R<u8> {
    u8::try_from(w).or_else(|_| broken("a Char word past a byte"))
}

#[cfg(test)]
mod tests {
    use crate::compile::Op;

    /// Ops that only move words, jump, reach the heap or mark the output an event keeps; every other op computes a value or
    /// decides a trap, and takes that from `ops`, so the engine never writes an operation the
    /// reference interpreter and the C++ read from there.
    fn plumbing(op: &Op) -> bool {
        matches!(
            op,
            Op::GetG { .. }
                | Op::PutG { .. }
                | Op::Retain { .. }
                | Op::Zero { .. }
                | Op::Restore { .. }
                | Op::Take { .. }
                | Op::Put { .. }
                | Op::Imm { .. }
                | Op::Addr { .. }
                | Op::Move { .. }
                | Op::Load { .. }
                | Op::Peek { .. }
                | Op::Drop { .. }
                | Op::Text { .. }
                | Op::At { .. }
                | Op::Enter { .. }
                | Op::Call { .. }
                | Op::Region { .. }
                | Op::Yield { .. }
                | Op::LendOut { .. }
                | Op::Jump { .. }
                | Op::JumpUnless { .. }
                | Op::JumpIf { .. }
                | Op::Switch { .. }
                | Op::Leave { .. }
                | Op::Check { .. }
                | Op::Out { .. }
                | Op::Showing { .. }
                | Op::Capture
                | Op::Flush
                | Op::Event { .. }
                | Op::RawEmpty { .. }
                | Op::InputLeft { .. }
        )
    }
    /// Not an operation: the IR broke its contract.
    fn not_an_op(op: &Op) -> bool {
        matches!(op, Op::Fail { .. })
    }
    /// The variants a `matches!` of fn `f` in this file names.
    fn named(f: &str) -> Vec<String> {
        let src = include_str!("exec.rs");
        let at = src.find(&format!("fn {f}(op: &Op)")).unwrap_or(0);
        let body = &src[at..];
        let body = &body[..body.find("\n    }\n").unwrap_or(body.len())];
        body.split("Op::")
            .skip(1)
            .map(|v| v.chars().take_while(char::is_ascii_alphanumeric).collect())
            .collect()
    }

    fn variants() -> Vec<String> {
        let src = include_str!("compile.rs");
        let start = src.find("pub(crate) enum Op {").unwrap_or(0);
        let body = &src[start..];
        let body = &body[..body.find("\n}\n").unwrap_or(body.len())];
        body.lines()
            .filter_map(|l| {
                let name = l.strip_prefix("    ")?;
                let end = name.find(|c: char| !c.is_ascii_alphanumeric())?;
                (name.starts_with(|c: char| c.is_ascii_uppercase()))
                    .then(|| name[..end].to_string())
            })
            .collect()
    }

    #[test]
    fn every_op_that_computes_takes_its_meaning_from_ops() {
        let src = include_str!("exec.rs");
        let start = src.find("fn exec(").unwrap_or(0);
        let end = src.find("\n#[cfg(test)]").unwrap_or(src.len());
        let body = &src[start..end];
        let arm_start = "                Op::";
        let mut arms = std::collections::HashMap::new();
        let mut name = String::new();
        for l in body.lines() {
            if let Some(rest) = l.strip_prefix(arm_start) {
                let n = rest
                    .find(|c: char| !c.is_ascii_alphanumeric())
                    .unwrap_or(rest.len());
                name = rest[..n].to_string();
            }
            if !name.is_empty() {
                arms.entry(name.clone())
                    .or_insert_with(String::new)
                    .push_str(l);
            }
        }
        let _typed: [fn(&Op) -> bool; 2] = [plumbing, not_an_op];
        let (plumbing, fail) = (named("plumbing"), named("not_an_op"));
        assert_eq!((plumbing.len(), fail.len()), (33, 1), "read the matches");
        let variants = variants();
        assert!(variants.len() > 30, "found the Op enum: {variants:?}");
        for v in &variants {
            if plumbing.contains(v) || fail.contains(v) {
                continue;
            }
            let arm = arms.get(v).map_or("", String::as_str);
            assert!(
                arm.contains("ops::"),
                "Op::{v} computes without `ops`: {arm}"
            );
        }
        for word in [
            "checked_",
            "wrapping_",
            "parse",
            "to_string",
            " as u8",
            "format!(\"{",
        ] {
            assert!(
                !body.contains(word),
                "exec.rs writes an operation itself: `{word}`"
            );
        }
    }
}
