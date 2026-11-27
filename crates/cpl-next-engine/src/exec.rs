//! Runs engine code. Registers are words of one stack; storage is counted heap objects,
//! copied when written while shared. Flows, lines and events follow the reference
//! interpreter's rules step for step.
use crate::compile::*;
use crate::layout::Lay;
use cpl_next_ir::ir::Func;
use cpl_next_ir::ops::{self, Fault};
use cpl_next_ir::prog::CTy;
use cpl_next_ir::run::{call_stack, Outcome, Trap};

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
    /// Its fn, the frame that called it and the line of the call: a trap's call stack.
    f: u32,
    parent: Option<usize>,
    line: u32,
    end: usize,
    region: Option<RegionAt>,
    pending: Option<Flow>,
}

#[derive(Clone, Copy)]
enum Loc {
    Stack(usize),
    Heap(u32, usize),
}

struct M<'a> {
    funcs: &'a [Func<CTy>],
    /// The frame whose code is running: a region body runs in its caller's.
    cur: usize,
    /// Each block run a trap left, innermost first: the block, its op and the frame.
    trail: Vec<(u32, usize, usize)>,
    c: &'a Code,
    stack: Vec<u64>,
    /// Object 0 is no object: a handle of 0 has none yet.
    heap: Vec<Obj>,
    free: Vec<u32>,
    frames: Vec<Frame>,
    out: String,
    events: Vec<String>,
    input: std::vec::IntoIter<String>,
    at: u32,
    texts: Vec<u32>,
    scratch: Vec<u32>,
}
type R<T> = Result<T, Trap>;

fn broken<T>(rule: &'static str) -> R<T> {
    Err(Fault::Internal(rule).into())
}

pub(crate) fn run(c: &Code, funcs: &[Func<CTy>], input: &str) -> Outcome {
    let mut m = M {
        c,
        funcs,
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
        out: String::new(),
        events: Vec::new(),
        input: input
            .split_whitespace()
            .map(String::from)
            .collect::<Vec<_>>()
            .into_iter(),
        at: 0,
        texts: vec![0; c.texts.len()],
        scratch: Vec::new(),
    };
    let fp = c.globals as usize;
    let trap = m.run_fn(c.main, fp, None).err().map(|t| Trap {
        line: m.at,
        stack: m.call_stack(),
        ..t
    });
    Outcome {
        events: m.events,
        output: m.out,
        trap,
    }
}

impl M<'_> {
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
        if !c.lays.owns[lay as usize] {
            return;
        }
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
        if !self.c.lays.owns[lay as usize] || times == 0 {
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
        if !c.lays.owns[lay as usize] {
            return;
        }
        if let Some(at) = &c.lays.flat[lay as usize] {
            for &o in at {
                let h = self.word(match loc {
                    Loc::Stack(a) => Loc::Stack(a + o as usize),
                    Loc::Heap(h, i) => Loc::Heap(h, i + o as usize),
                });
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
        match loc {
            Loc::Stack(a) => self.c.lays.handles(lay, &self.stack[a..], &mut hs),
            Loc::Heap(h, i) => self
                .c
                .lays
                .handles(lay, &self.heap[h as usize].words[i..], &mut hs),
        }
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
            if self.c.lays.owns[elem as usize] {
                let stride = self.c.lays.size[elem as usize] as usize;
                for i in 0..len {
                    self.c.lays.handles(elem, &words[i * stride..], &mut hs);
                }
            }
            self.free.push(h);
        }
    }
    fn word(&self, loc: Loc) -> u64 {
        match loc {
            Loc::Stack(a) => self.stack[a],
            Loc::Heap(h, i) => self.heap[h as usize].words[i],
        }
    }
    fn set_word(&mut self, loc: Loc, v: u64) {
        match loc {
            Loc::Stack(a) => self.stack[a] = v,
            Loc::Heap(h, i) => self.heap[h as usize].words[i] = v,
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
        match loc {
            Loc::Stack(s) => self.copy(s, a, n),
            Loc::Heap(h, i) => {
                self.stack[a..a + n].copy_from_slice(&self.heap[h as usize].words[i..i + n])
            }
        }
    }
    /// Copies `n` words at stack address `a` to `loc`.
    fn write(&mut self, loc: Loc, a: usize, n: usize) {
        match loc {
            Loc::Stack(d) => self.copy(a, d, n),
            Loc::Heap(h, i) => {
                self.heap[h as usize].words[i..i + n].copy_from_slice(&self.stack[a..a + n])
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
    fn loc(&mut self, fp: usize, p: u32) -> R<Loc> {
        let pc = &self.c.places[p as usize];
        let mut loc = match pc.root {
            Root::Local(r) => Loc::Stack(fp + r as usize),
            Root::Global(g) => Loc::Stack(g as usize),
        };
        for st in &pc.steps {
            loc = match *st {
                Step::Off(at) => match loc {
                    Loc::Stack(a) => Loc::Stack(a + at as usize),
                    Loc::Heap(h, i) => Loc::Heap(h, i + at as usize),
                },
                Step::Deref => Loc::Stack(self.word(loc) as usize),
                Step::Slot { i, stride } => {
                    let idx = self.stack[fp + i as usize] as i64;
                    let h = self.unique(loc)?;
                    match usize::try_from(idx) {
                        Ok(idx) if idx < self.heap[h as usize].len => {
                            Loc::Heap(h, idx * stride as usize)
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
        let fc = &self.c.funcs[f as usize];
        // Spec 9.6: no depth counting in the language; this stands for the 1 GB stack running out.
        if self.frames.len() >= 4_000_000 {
            return Err(Fault::StackExhausted.into());
        }
        let end = fp + fc.frame as usize;
        if self.stack.len() < end {
            self.stack.resize(end.max(self.stack.len() * 2), 0);
        }
        let parent = (!self.frames.is_empty()).then_some(self.cur);
        self.frames.push(Frame {
            fp,
            f,
            parent,
            line: self.at,
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
            self.release(lay, Loc::Stack(fp + at as usize));
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
    /// it, from the chain of the op it stopped at, with the line running in it.
    fn call_stack(&self) -> Vec<String> {
        let mut calls = Vec::new();
        let (mut k, mut line, mut pos) = (Some(self.cur), self.at, 0);
        while let Some(fr) = k.and_then(|i| self.frames.get(i)) {
            if let Some(i) = self.trail[pos..].iter().position(|t| Some(t.2) == k) {
                let (b, pc, _) = self.trail[pos + i];
                pos += i + 1;
                let marks = &self.c.marks[b as usize];
                let at = marks
                    .partition_point(|m| m.0 as usize <= pc)
                    .saturating_sub(1);
                let chain = marks
                    .get(at)
                    .map_or(&[][..], |m| &self.c.chains[m.1 as usize]);
                for &(g, save) in chain.iter().rev() {
                    calls.push((g as usize, line));
                    line = self.stack[fr.fp + save as usize] as u32;
                }
            }
            calls.push((fr.f as usize, line));
            (k, line) = (fr.parent, fr.line);
        }
        call_stack(self.funcs, calls)
    }
    fn enter(&mut self, line: u32) -> u32 {
        if line != 0 {
            self.at = line;
        }
        self.at
    }

    fn exec(&mut self, b: u32, fr: usize) -> R<Flow> {
        let mut pc = 0;
        let flow = self.run_block(b, fr, &mut pc);
        if flow.is_err() {
            self.trail.push((b, pc.saturating_sub(1), fr));
        }
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
                    let loc = self.loc(fp, p)?;
                    self.read(loc, r(d), self.size(lay));
                    self.retain(lay, r(d));
                }
                Op::Peek { d, p } => {
                    let loc = self.loc(fp, p)?;
                    self.stack[r(d)] = self.word(loc);
                }
                Op::Store { p, s, lay } => {
                    let loc = self.loc(fp, p)?;
                    self.release(lay, loc);
                    self.write(loc, r(s), self.size(lay));
                }
                Op::Drop { s, lay } => self.release(lay, Loc::Stack(r(s))),
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
                Op::Float { op, d, a, b } => {
                    let (x, y) = (
                        f64::from_bits(self.stack[r(a)]),
                        f64::from_bits(self.stack[r(b)]),
                    );
                    self.stack[r(d)] = ops::float(op, x, y)?.to_bits();
                }
                Op::FNeg { d, a } => {
                    self.stack[r(d)] = ops::neg_float(f64::from_bits(self.stack[r(a)])).to_bits()
                }
                Op::ToF64 { d, a } => {
                    self.stack[r(d)] = ops::f64_of_int(self.stack[r(a)] as i64).to_bits()
                }
                Op::Neg { d, a } => self.stack[r(d)] = ops::neg(self.stack[r(a)] as i64)? as u64,
                Op::Not { d, a } => self.stack[r(d)] = u64::from(ops::not(self.stack[r(a)] != 0)),
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
                Op::At { line } => self.at = line,
                Op::Enter { line, save } => self.stack[r(save)] = u64::from(self.enter(line)),
                Op::Call { c: k } => {
                    self.call(k, fr, None)?;
                    if let Some(save) = c.calls[k as usize].save {
                        self.at = self.stack[r(save)] as u32;
                    }
                }
                Op::Region {
                    c: k,
                    body,
                    var,
                    lend,
                    jumps,
                } => {
                    let here = self.enter(c.calls[k as usize].line);
                    let exit = self.call(k, fr, Some(RegionAt { fr, body, var }))?;
                    self.at = here;
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
                    self.release(lay, Loc::Stack(v));
                    let n = self.size(lay);
                    self.stack.copy_within(r(s)..r(s) + n, v);
                    let (here, was) = (self.at, self.cur);
                    self.cur = rg.fr;
                    let flow = self.exec(rg.body, rg.fr)?;
                    (self.at, self.cur) = (here, was);
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
                    self.release(lay, Loc::Stack(v));
                    self.read(loc, v, n);
                    let (here, was) = (self.at, self.cur);
                    self.cur = rg.fr;
                    let flow = self.exec(rg.body, rg.fr)?;
                    (self.at, self.cur) = (here, was);
                    let loc = self.loc(fp, p)?;
                    self.write(loc, v, n);
                    self.stack[v..v + n].fill(0);
                    // The rest of the lender still runs; the caller leaves after it.
                    if !matches!(flow, Flow::Normal) {
                        self.frames[fr].pending = Some(flow);
                    }
                }
                Op::Jump { t } => *pc = t as usize,
                Op::JumpUnless { c: x, t } => {
                    if self.stack[r(x)] == 0 {
                        *pc = t as usize;
                    }
                }
                Op::JumpIf { c: x, t } => {
                    if self.stack[r(x)] != 0 {
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
                    line,
                    msg,
                } => {
                    if self.stack[r(x)] == 0 {
                        self.enter(line);
                        return Err(Trap {
                            msg: c.msgs[msg as usize].clone(),
                            ..Fault::Check(kind).into()
                        });
                    }
                }
                Op::Guard { op, c: x } => ops::check(op, self.stack[r(x)] != 0)?,
                Op::Print { s, kind, fmt } => {
                    if let Some(v) = scalar(self.stack[r(s)], kind) {
                        ops::show_fmt(v, fmt, &mut self.out);
                    }
                }
                Op::PrintText { t } => self.out.push_str(&c.texts[t as usize]),
                Op::Event { e } => {
                    let ev = &c.events[e as usize];
                    let mut line = format!("ev {}", ev.label);
                    for &(s, kind, lay) in &ev.vals {
                        line.push(' ');
                        match scalar(self.stack[r(s)], kind) {
                            Some(v) => ops::show(v, &mut line),
                            None => {
                                line.push('?');
                                self.release(lay, Loc::Stack(r(s)));
                            }
                        }
                    }
                    self.events.push(line);
                }
                Op::RawEmpty { d, elem } => {
                    self.stack[r(d)] = u64::from(self.alloc(elem, 0, Vec::new()))
                }
                Op::RawNew { d, n, fill, elem } => {
                    let len = ops::storage_size(self.stack[r(n)] as i64)?;
                    let stride = self.size(elem);
                    let cell = &self.stack[r(fill)..r(fill) + stride];
                    let words = if stride == 1 {
                        vec![cell[0]; len]
                    } else {
                        cell.repeat(len)
                    };
                    if c.lays.owns[elem as usize] {
                        let mut hs = Vec::new();
                        c.lays.handles(elem, cell, &mut hs);
                        for h in &hs {
                            self.heap[*h as usize].rc += len as u32;
                        }
                        self.dec(hs);
                    }
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
                    self.read(Loc::Heap(h, at), r(d), self.size(elem));
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
                    self.release(elem, Loc::Heap(h, at));
                    self.write(Loc::Heap(h, at), r(x), self.size(elem));
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
                    self.read(Loc::Heap(h, at), r(d), size);
                    self.heap[h as usize].words[at..at + size].fill(0);
                }
                Op::RawCap { d, n, elem } => {
                    let len = ops::storage_size(self.stack[r(n)] as i64)?;
                    let words = vec![0; len * self.size(elem)];
                    self.stack[r(d)] = u64::from(self.alloc(elem, len, words));
                }
                Op::InputLeft { d } => {
                    self.stack[r(d)] = u64::from(!self.input.as_slice().is_empty())
                }
                Op::ReadInt { d } => {
                    let t = self.input.next();
                    let v =
                        ops::read_int(t.as_deref()).map_err(|f| Trap::at_token(f, t.as_deref()))?;
                    self.stack[r(d)] = v as u64;
                }
                Op::ToU64 { d, a } => self.stack[r(d)] = ops::u64_of_int(self.stack[r(a)] as i64),
                Op::FromU64 { d, a } => {
                    self.stack[r(d)] = ops::int_of_u64(self.stack[r(a)])? as u64
                }
                Op::CharOf { d, a } => {
                    self.stack[r(d)] = u64::from(ops::char_of(self.stack[r(a)] as i64)?)
                }
                Op::CodeOf { d, a } => {
                    self.stack[r(d)] = ops::code_of(byte(self.stack[r(a)])?) as u64
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

/// A word of a value of `kind` as the scalar it holds.
fn scalar(w: u64, kind: Kind) -> Option<ops::Scalar> {
    Some(match kind {
        Kind::Int => ops::Scalar::Int(w as i64),
        Kind::U64 => ops::Scalar::U64(w),
        Kind::F64 => ops::Scalar::F64(w),
        Kind::Bool => ops::Scalar::Bool(w != 0),
        Kind::Char => ops::Scalar::Char(byte(w).ok()?),
        Kind::Other => return None,
    })
}
/// A `Char`'s word as its byte.
fn byte(w: u64) -> R<u8> {
    u8::try_from(w).or_else(|_| broken("a Char word past a byte"))
}

#[cfg(test)]
mod tests {
    /// Ops that only move words, jump or reach the heap; every other op computes a value or
    /// decides a trap, and takes that from `ops`, so the engine never writes an operation the
    /// reference interpreter and the C++ read from there.
    const PLUMBING: [&str; 30] = [
        "GetG",
        "PutG",
        "Retain",
        "Zero",
        "Restore",
        "Take",
        "Put",
        "Imm",
        "Addr",
        "Move",
        "Load",
        "Peek",
        "Store",
        "Drop",
        "Text",
        "At",
        "Enter",
        "Call",
        "Region",
        "Yield",
        "LendOut",
        "Jump",
        "JumpUnless",
        "JumpIf",
        "Switch",
        "Leave",
        "Check",
        "PrintText",
        "RawEmpty",
        "InputLeft",
    ];
    /// Not an operation: the IR broke its contract.
    const FAIL: &str = "Fail";

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
        let end = src.find("/// A word of a value of").unwrap_or(src.len());
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
        let variants = variants();
        assert!(variants.len() > 30, "found the Op enum: {variants:?}");
        for v in &variants {
            if PLUMBING.contains(&v.as_str()) || v == FAIL {
                continue;
            }
            let arm = arms.get(v).map_or("", String::as_str);
            assert!(
                arm.contains("ops::"),
                "Op::{v} computes without `ops`: {arm}"
            );
        }
        for p in PLUMBING {
            assert!(
                variants.iter().any(|v| v == p),
                "Op::{p} is listed but gone"
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
