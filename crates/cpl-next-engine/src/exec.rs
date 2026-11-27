//! Runs engine code. Registers are words of one stack; storage is counted heap objects,
//! copied when written while shared. Flows, lines and events follow the reference
//! interpreter's rules step for step.
use crate::compile::*;
use crate::layout::Lay;
use cpl_next_ir::ir::Site;
use cpl_next_ir::ops::{self, Fault};
use cpl_next_ir::run::{Outcome, Trap};

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
    c: &'a Code,
    stack: Vec<u64>,
    /// Object 0 is no object: a handle of 0 has none yet.
    heap: Vec<Obj>,
    free: Vec<u32>,
    frames: Vec<Frame>,
    out: String,
    events: Vec<String>,
    input: std::vec::IntoIter<String>,
    at: Site,
    texts: Vec<u32>,
    scratch: Vec<u32>,
}
type R<T> = Result<T, Trap>;

fn broken<T>(rule: &'static str) -> R<T> {
    Err(Fault::Internal(rule).into())
}

pub(crate) fn run(c: &Code, input: &str) -> Outcome {
    let mut m = M {
        c,
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
    let trap = m
        .run_fn(c.main, fp, None)
        .err()
        .map(|t| Trap { line: m.at, ..t });
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
        if !self.c.lays.owns[lay as usize] {
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
        if !self.c.lays.owns[lay as usize] {
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
    /// Copies `n` words at `loc` to stack address `a`.
    fn read(&mut self, loc: Loc, a: usize, n: usize) {
        match loc {
            Loc::Stack(s) => self.stack.copy_within(s..s + n, a),
            Loc::Heap(h, i) => {
                self.stack[a..a + n].copy_from_slice(&self.heap[h as usize].words[i..i + n])
            }
        }
    }
    /// Copies `n` words at stack address `a` to `loc`.
    fn write(&mut self, loc: Loc, a: usize, n: usize) {
        match loc {
            Loc::Stack(d) => self.stack.copy_within(a..a + n, d),
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
        self.frames.push(Frame {
            fp,
            end,
            region,
            pending: None,
        });
        let fr = self.frames.len() - 1;
        let flow = self.exec(fc.block, fr)?;
        let Some(frame) = self.frames.pop() else {
            return broken("a call returned with no frame");
        };
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
        self.stack[top..end].fill(0);
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
    fn enter(&mut self, site: Site) -> Site {
        if site != 0 {
            self.at = site;
        }
        self.at
    }
    fn show(&mut self, w: u64, kind: Kind) -> Option<String> {
        Some(match kind {
            Kind::Int => (w as i64).to_string(),
            Kind::U64 => w.to_string(),
            Kind::Bool => (if w != 0 { "true" } else { "false" }).to_string(),
            Kind::Char => char::from(w as u8).to_string(),
            Kind::Other => return None,
        })
    }

    fn exec(&mut self, b: u32, fr: usize) -> R<Flow> {
        let c = self.c;
        let ops = &c.blocks[b as usize];
        let fp = self.frames[fr].fp;
        let mut pc = 0;
        while let Some(&op) = ops.get(pc) {
            pc += 1;
            let r = |x: Reg| fp + x as usize;
            match op {
                Op::Imm { d, v } => self.stack[r(d)] = v,
                Op::Addr { d, s } => self.stack[r(d)] = r(s) as u64,
                Op::Move { d, s, n } => self.stack.copy_within(r(s)..r(s) + n as usize, r(d)),
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
                Op::Neg { d, a } => self.stack[r(d)] = ops::neg(self.stack[r(a)] as i64)? as u64,
                Op::Not { d, a } => self.stack[r(d)] = u64::from(self.stack[r(a)] == 0),
                Op::Cmp { op, d, a, b, kind } => {
                    let (x, y) = (self.stack[r(a)], self.stack[r(b)]);
                    let (x, y) = match kind {
                        Kind::Int => (i128::from(x as i64), i128::from(y as i64)),
                        _ => (i128::from(x), i128::from(y)),
                    };
                    self.stack[r(d)] = u64::from(ops::compare(op, x, y)?);
                }
                Op::At { site } => self.at = site,
                Op::Enter { site, save } => self.stack[r(save)] = u64::from(self.enter(site)),
                Op::Call { c: k } => {
                    self.call(k, fr, None)?;
                    if let Some(save) = c.calls[k as usize].save {
                        self.at = self.stack[r(save)] as Site;
                    }
                }
                Op::Region {
                    c: k,
                    body,
                    var,
                    lend,
                    jumps,
                } => {
                    let here = self.enter(c.calls[k as usize].site);
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
                    pc = to as usize;
                }
                Op::Yield { s, lay } => {
                    let Some(rg) = self.frames[fr].region else {
                        return broken("yield with no caller code");
                    };
                    let v = self.frames[rg.fr].fp + rg.var as usize;
                    self.release(lay, Loc::Stack(v));
                    let n = self.size(lay);
                    self.stack.copy_within(r(s)..r(s) + n, v);
                    let here = self.at;
                    let flow = self.exec(rg.body, rg.fr)?;
                    self.at = here;
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
                    let here = self.at;
                    let flow = self.exec(rg.body, rg.fr)?;
                    self.at = here;
                    let loc = self.loc(fp, p)?;
                    self.write(loc, v, n);
                    self.stack[v..v + n].fill(0);
                    // The rest of the lender still runs; the caller leaves after it.
                    if !matches!(flow, Flow::Normal) {
                        self.frames[fr].pending = Some(flow);
                    }
                }
                Op::Jump { t } => pc = t as usize,
                Op::JumpUnless { c: x, t } => {
                    if self.stack[r(x)] == 0 {
                        pc = t as usize;
                    }
                }
                Op::Switch { c: x, table } => {
                    match c.tables[table as usize].get(self.stack[r(x)] as usize) {
                        Some(&t) => pc = t as usize,
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
                    site,
                    msg,
                } => {
                    if self.stack[r(x)] == 0 {
                        self.enter(site);
                        return Err(Trap {
                            fault: Fault::Check(kind),
                            line: 0,
                            msg: c.msgs[msg as usize].clone(),
                        });
                    }
                }
                Op::Guard { op, c: x } => ops::check(op, self.stack[r(x)] != 0)?,
                Op::Print { s, kind } => {
                    if let Some(t) = self.show(self.stack[r(s)], kind) {
                        self.out.push_str(&t);
                    }
                }
                Op::PrintText { t } => self.out.push_str(&c.texts[t as usize]),
                Op::Event { e } => {
                    let ev = &c.events[e as usize];
                    let mut line = format!("ev {}", ev.label);
                    for &(s, kind, lay) in &ev.vals {
                        line.push(' ');
                        match self.show(self.stack[r(s)], kind) {
                            Some(t) => line.push_str(&t),
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
                Op::InputLeft { d } => {
                    self.stack[r(d)] = u64::from(!self.input.as_slice().is_empty())
                }
                Op::ReadInt { d } => {
                    match self
                        .input
                        .next()
                        .and_then(|t| t.parse::<i64>().ok())
                        .filter(|v| *v != i64::MIN)
                    {
                        Some(v) => self.stack[r(d)] = v as u64,
                        None => return Err(Fault::BadIntToken.into()),
                    }
                }
                Op::ToU64 { d, a } => self.stack[r(d)] = self.stack[r(a)],
                Op::FromU64 { d, a } => {
                    self.stack[r(d)] = ops::int_of_u64(self.stack[r(a)])? as u64
                }
                Op::CharOf { d, a } => self.stack[r(d)] = u64::from(self.stack[r(a)] as u8),
                Op::Fail { rule } => return broken(rule),
            }
        }
        Ok(Flow::Normal)
    }
}
