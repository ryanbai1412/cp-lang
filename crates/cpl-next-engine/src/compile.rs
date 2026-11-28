//! Canonical IR to engine code: each fn becomes a block of `Op`s over word registers of its
//! frame (its result, then its locals, then temporaries); each region body is a block of its
//! own, run in the frame of the fn it is written in.
use crate::layout::{Lay, Layouts};
use cpl_next_ir::ir::*;
use cpl_next_ir::ops;
use cpl_next_ir::prog::{CTy, Program};
use cpl_next_ir::run::{Limits, Stack};

/// A word offset in the running frame.
pub(crate) type Reg = u32;
/// No jump target: the flow leaves the block.
pub(crate) const LEAVE: u32 = u32::MAX;
/// An origin no op of the block set: the origin of the block run outside it.
pub(crate) const INHERIT: u32 = u32::MAX;
/// A call origin kept in its chain register, set by a block outside.
pub(crate) const SAVED: u32 = u32::MAX - 1;

/// How a word prints, compares and shows in an event.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Kind {
    Int,
    Bool,
    Char,
    U64,
    F64,
    Other,
}
fn kind(t: &CTy) -> Kind {
    match t {
        CTy::Int | CTy::Dom(_) => Kind::Int,
        CTy::Bool => Kind::Bool,
        CTy::Char => Kind::Char,
        CTy::U64 => Kind::U64,
        CTy::F64 => Kind::F64,
        CTy::Lent(t) => kind(t),
        _ => Kind::Other,
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Leave {
    Break,
    Continue,
    Return,
}

/// Where a flow that a region hands back goes in the block: a jump target or `LEAVE`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Jumps {
    pub brk: u32,
    pub cont: u32,
    pub other: u32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Op {
    Imm {
        d: Reg,
        v: u64,
    },
    /// The stack address of a register of this frame.
    Addr {
        d: Reg,
        s: Reg,
    },
    /// Words from register to register; what they own goes with them.
    Move {
        d: Reg,
        s: Reg,
        n: u32,
    },
    /// A copy of a place's value: it counts the storage it shares.
    Load {
        d: Reg,
        p: u32,
        lay: Lay,
    },
    /// One word of a place, not counted: the storage a read looks at.
    Peek {
        d: Reg,
        p: u32,
    },
    Drop {
        s: Reg,
        lay: Lay,
    },
    Text {
        d: Reg,
        t: u32,
    },
    Int {
        op: Intr,
        d: Reg,
        a: Reg,
        b: Reg,
    },
    Word {
        op: Intr,
        d: Reg,
        a: Reg,
        b: Reg,
    },
    /// An `I128` operation. `I128`s are two words, low first: `d`, `a`, and `b` unless `op`
    /// is a shift, whose amount is an `Int`.
    /// `b` is one word when `narrow` (a shift count).
    Wide {
        op: Intr,
        d: Reg,
        a: Reg,
        b: Reg,
        narrow: bool,
    },
    /// A scalar operation of one operand, which `ops::unary` computes.
    Unary {
        op: Intr,
        sig: ops::Sig,
        d: Reg,
        a: Reg,
    },
    /// A comparison of two `I128`s.
    WideCmp {
        op: Intr,
        d: Reg,
        a: Reg,
        b: Reg,
    },
    Cmp {
        op: Intr,
        d: Reg,
        a: Reg,
        b: Reg,
        kind: Kind,
    },
    At {
        origin: u32,
    },
    /// A call's origin starts running: `save` keeps the origin to go back to after it.
    Enter {
        origin: u32,
        save: Reg,
    },
    Call {
        c: u32,
    },
    Region {
        c: u32,
        body: u32,
        var: Reg,
        lend: bool,
        jumps: Jumps,
    },
    Yield {
        s: Reg,
        lay: Lay,
    },
    LendOut {
        p: u32,
        lay: Lay,
    },
    Jump {
        t: u32,
    },
    JumpUnless {
        c: Reg,
        t: u32,
    },
    JumpIf {
        c: Reg,
        t: u32,
    },
    Switch {
        c: Reg,
        table: u32,
    },
    Leave(Leave),
    Check {
        c: Reg,
        kind: CheckKind,
        origin: u32,
        msg: u32,
    },
    Guard {
        op: Intr,
        c: Reg,
    },
    Out {
        r: Reg,
        n: Reg,
        owned: bool,
    },
    ParseF64 {
        d: Reg,
        r: Reg,
        n: Reg,
        owned: bool,
    },
    /// Whether an event is showing a value now: the code that shows it places no events.
    Showing {
        d: Reg,
    },
    /// Where the output is now: the next event keeps what is written after it.
    Capture,
    /// Hands the output written so far to the judge (R10.4).
    Flush,
    Event {
        e: u32,
    },
    RawEmpty {
        d: Reg,
        elem: Lay,
    },
    RawNew {
        d: Reg,
        n: Reg,
        fill: Reg,
        elem: Lay,
    },
    /// Cell `i` of the storage whose handle is in `r`; `owned`: the handle is dropped after.
    RawGet {
        d: Reg,
        r: Reg,
        i: Reg,
        elem: Lay,
        owned: bool,
    },
    RawSet {
        p: u32,
        i: Reg,
        x: Reg,
        elem: Lay,
    },
    /// Cell `i` of the storage at place `p`, moved to `d` and left unset.
    RawTake {
        d: Reg,
        p: u32,
        i: Reg,
        elem: Lay,
    },
    RawCap {
        d: Reg,
        n: Reg,
        elem: Lay,
    },
    InputLeft {
        d: Reg,
    },
    ReadInt {
        d: Reg,
    },
    ReadByte {
        d: Reg,
    },
    PeekByte {
        d: Reg,
        k: Reg,
    },
    RandWord {
        d: Reg,
    },
    Float {
        op: Intr,
        d: Reg,
        a: Reg,
        b: Reg,
    },
    Fail {
        rule: &'static str,
    },
    /// `n` words of the globals at stack address `a`, read and written.
    GetG {
        d: Reg,
        a: u32,
        n: u32,
    },
    PutG {
        a: u32,
        s: Reg,
        n: u32,
    },
    /// The value in `s` gains a holder.
    Retain {
        s: Reg,
        lay: Lay,
    },
    /// Words an inlined fn's owned locals start as.
    Zero {
        d: Reg,
        n: u32,
    },
    /// An inlined call is over: its origin is the caller's again.
    Restore {
        save: Reg,
    },
    /// A lent place's words move to an inlined fn's parameter, and back with `Put`.
    Take {
        d: Reg,
        p: u32,
        n: u32,
    },
    Put {
        p: u32,
        s: Reg,
        n: u32,
    },
}

#[derive(Clone, Copy)]
enum Spot {
    Reg(Reg),
    Global(u32),
    Place(u32),
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum Root {
    Local(Reg),
    Global(u32),
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum Step {
    Off(u32),
    /// The word here is a stack address.
    Deref,
    /// The word here is a handle; go to the cell the register indexes.
    Slot {
        i: Reg,
        stride: u32,
    },
}
#[derive(Clone, Debug)]
pub(crate) struct PlaceCode {
    pub root: Root,
    pub steps: Vec<Step>,
}

#[derive(Clone, Debug)]
pub(crate) enum ArgCode {
    /// An evaluated argument in a register, moved to the parameter.
    Move { s: Reg, n: u32 },
    /// A lent place: its value moves to the parameter and back after the call.
    Lend { p: u32 },
}
#[derive(Clone, Debug)]
pub(crate) struct CallCode {
    pub f: u32,
    pub origin: u32,
    pub args: Vec<ArgCode>,
    pub d: Reg,
}
pub(crate) struct FnCode {
    pub block: u32,
    pub frame: u32,
    pub ret: u32,
    /// Each parameter's register and size.
    pub params: Vec<(Reg, u32)>,
    pub lent: Vec<bool>,
    /// The words of its locals and result, before temporaries.
    pub locals: u32,
    /// What the frame releases when the fn ends (`Func::drops`).
    pub drops: Vec<(Reg, Lay)>,
}
pub(crate) struct EventCode {
    pub kind: cpl_next_ir::ir::EventKind,
    pub func: u32,
    pub site: cpl_next_ir::ir::Site,
    /// How many values it shows, each the text written since its `Capture`.
    pub vals: usize,
}

pub(crate) struct Code {
    /// The inlined calls an op runs in, outermost first: the fn and the register holding the
    /// origin it was called from. Chain 0 is none.
    pub chains: Vec<Vec<(u32, Reg)>>,
    /// Per block: from which op on each chain holds, in order.
    pub marks: Vec<Vec<(u32, u32)>>,
    /// Per block whose origins every way through agrees on: each op's origin and the call origins of
    /// its chain (`saved`). A trap reads them here; a block with none sets `at` as it runs.
    pub origins: Vec<Vec<(u32, u32)>>,
    /// Call origins of a chain, outermost first: an origin, `INHERIT`, or `SAVED`.
    pub saved: Vec<Vec<u32>>,
    pub blocks: Vec<Vec<Op>>,
    pub funcs: Vec<FnCode>,
    pub places: Vec<PlaceCode>,
    pub calls: Vec<CallCode>,
    pub texts: Vec<String>,
    pub msgs: Vec<String>,
    pub events: Vec<EventCode>,
    pub tables: Vec<Vec<u32>>,
    pub lays: Layouts,
    pub globals: u32,
    /// What each run's calls take of its stack (`run::Stack`).
    pub stack: Stack,
    /// Where each global's words start.
    pub global_at: Vec<u32>,
    /// Each global's layout.
    pub global_lays: Vec<Lay>,
    pub main: u32,
    /// The layout of a string literal's characters.
    pub char_lay: Lay,
}

pub(crate) fn compile(p: &Program, funcs: &[Func<CTy>]) -> Code {
    let mut c = Code {
        chains: vec![Vec::new()],
        marks: Vec::new(),
        origins: Vec::new(),
        global_at: Vec::new(),
        global_lays: Vec::new(),
        saved: vec![Vec::new()],
        blocks: Vec::new(),
        funcs: Vec::new(),
        places: Vec::new(),
        calls: Vec::new(),
        texts: Vec::new(),
        msgs: Vec::new(),
        events: Vec::new(),
        tables: Vec::new(),
        lays: Layouts::default(),
        globals: 0,
        stack: Stack::default(),
        main: p.main as u32,
        char_lay: 0,
    };
    c.char_lay = c.lays.of(p, &CTy::Char);
    let mut global_at = Vec::with_capacity(p.globals.len());
    for g in &p.globals {
        global_at.push(c.globals);
        let l = c.lays.of(p, g);
        c.globals += c.lays.size[l as usize];
        c.global_lays.push(l);
    }
    c.global_at = global_at.clone();
    c.stack = Stack::new(p, Limits::default());
    // Every fn's frame shape first: a call needs its callee's.
    let mut offs = Vec::with_capacity(funcs.len());
    for f in funcs {
        let ret = c.lays.of(p, &f.ret);
        let mut at = c.lays.size[ret as usize];
        let mut off = Vec::with_capacity(f.locals.len());
        for t in &f.locals {
            off.push(at);
            let l = c.lays.of(p, t);
            at += c.lays.size[l as usize];
        }
        let mut drops = Vec::with_capacity(f.drops.len());
        for &l in &f.drops {
            drops.push((off[l], c.lays.of(p, &f.locals[l])));
        }
        c.funcs.push(FnCode {
            block: 0,
            frame: at,
            ret: c.lays.size[ret as usize],
            params: f
                .params
                .iter()
                .map(|(l, _)| {
                    let lay = c.lays.of(p, &f.locals[*l]);
                    (off[*l], c.lays.size[lay as usize])
                })
                .collect(),
            lent: f.params.iter().map(|(_, lent)| *lent).collect(),
            locals: at,
            drops,
        });
        offs.push(off);
    }
    for (i, f) in funcs.iter().enumerate() {
        let frame = c.funcs[i].frame;
        let k = if loops(&f.body) { CONSTS } else { 0 };
        let mut fc = Fc {
            c: &mut c,
            p,
            f,
            funcs,
            offs: &offs,
            off: offs[i].clone(),
            global_at: &global_at,
            top: frame + k,
            max: frame + k,
            ops: (0..k).map(|j| Op::Imm { d: frame + j, v: 0 }).collect(),
            loops: Vec::new(),
            ret: 0,
            exits: None,
            inlining: vec![i],
            chain: 0,
            marks: vec![(0, 0)],
            lending: None,
            looping: 0,
            consts: Vec::new(),
            kregs: (frame, k),
        };
        fc.block(&f.body);
        // Each constant a loop asked for set at entry; the rest of the first ops become jumps
        // to the next op, which `tidy` drops.
        for j in 0..k as usize {
            fc.ops[j] = match fc.consts.get(j) {
                Some(&v) => Op::Imm {
                    d: frame + j as u32,
                    v,
                },
                None => Op::Jump { t: j as u32 + 1 },
            };
        }
        let (ops, max) = (std::mem::take(&mut fc.ops), fc.max);
        let marks = std::mem::take(&mut fc.marks);
        c.funcs[i].frame = max;
        c.funcs[i].block = c.blocks.len() as u32;
        c.blocks.push(ops);
        c.marks.push(marks);
    }
    crate::peep::tidy(&mut c);
    c
}

enum Ctx {
    /// Jumps to patch: `break` to the end, `continue` to the step.
    Loop { brk: Vec<usize>, cont: Vec<usize> },
    /// A loop's step: a flow that leaves it ends the step, as the reference interpreter does.
    Step { end: Vec<usize> },
}

struct Fc<'a> {
    c: &'a mut Code,
    p: &'a Program,
    f: &'a Func<CTy>,
    funcs: &'a [Func<CTy>],
    offs: &'a [Vec<Reg>],
    off: Vec<Reg>,
    global_at: &'a [u32],
    top: Reg,
    max: Reg,
    ops: Vec<Op>,
    loops: Vec<Ctx>,
    /// Where `return` puts its value: the result words, or an inlined call's destination.
    ret: Reg,
    /// In an inlined fn: the jumps a flow out of it takes to its end.
    exits: Option<Vec<usize>>,
    /// The fns being compiled, the outermost first: none is inlined into itself.
    inlining: Vec<usize>,
    /// The chain of inlined calls being compiled (`Code::chains`), and where `ops` enters each.
    chain: u32,
    marks: Vec<(u32, u32)>,
    /// In an inlined lender: the caller's region body, which its `lend` runs in place.
    lending: Option<Lending<'a>>,
    /// How many loops the op being compiled is in.
    looping: u32,
    /// Constants a loop reads, each in a register its fn's entry sets (the first ops).
    consts: Vec<u64>,
    /// The first register for constants and how many there are.
    kregs: (Reg, u32),
}

/// How many constants a fn with a loop keeps in registers.
const CONSTS: u32 = 8;

fn loops(b: &[Stmt<CTy>]) -> bool {
    b.iter().any(|s| match s {
        Stmt::Loop(..) => true,
        Stmt::If(_, x, y) => loops(x) || loops(y),
        Stmt::Switch(_, arms) => arms.iter().any(|a| loops(a)),
        Stmt::Region { body, .. } => loops(body),
        _ => false,
    })
}

/// A region body an inlined lender runs where it lends: the caller's context to compile it in.
struct Lending<'a> {
    var: Reg,
    body: Vec<Stmt<CTy>>,
    off: Vec<Reg>,
    f: &'a Func<CTy>,
    chain: u32,
    /// The lending of the caller, when it is itself an inlined lender.
    outer: Option<Box<Lending<'a>>>,
}

impl Fc<'_> {
    fn lay(&mut self, t: &CTy) -> Lay {
        self.c.lays.of(self.p, t)
    }
    fn size(&mut self, t: &CTy) -> u32 {
        let l = self.lay(t);
        self.c.lays.size[l as usize]
    }
    /// Whether `r` is a word of one of the fn's locals, which an expression may read.
    fn local_at(&mut self, r: Reg) -> bool {
        let f = self.f;
        (0..f.locals.len()).any(|l| {
            let (o, n) = (self.off[l], self.size(&f.locals[l]));
            (o..o + n).contains(&r)
        })
    }
    fn temp(&mut self, n: u32) -> Reg {
        let r = self.top;
        self.top += n;
        self.max = self.max.max(self.top);
        r
    }
    fn here(&self) -> u32 {
        self.ops.len() as u32
    }
    fn patch(&mut self, at: usize, t: u32) {
        match &mut self.ops[at] {
            Op::Jump { t: x } | Op::JumpUnless { t: x, .. } | Op::JumpIf { t: x, .. } => *x = t,
            _ => {
                self.ops[at] = Op::Fail {
                    rule: "the engine patched a jump that is not one",
                }
            }
        }
    }
    fn fail(&mut self, rule: &'static str) {
        self.ops.push(Op::Fail { rule });
    }

    fn block(&mut self, b: &[Stmt<CTy>]) {
        for s in b {
            let top = self.top;
            self.stmt(s);
            self.top = top;
        }
    }
    /// A flow out of the innermost loop or step: a jump, or leaving the block.
    fn leave(&mut self, how: Leave) {
        let at = self.ops.len();
        match (self.loops.last_mut(), how) {
            (Some(Ctx::Loop { brk, .. }), Leave::Break) => brk.push(at),
            (Some(Ctx::Loop { cont, .. }), Leave::Continue) => cont.push(at),
            (Some(Ctx::Step { end }), _) => end.push(at),
            _ if self.exits.is_some() => {
                if let Some(exits) = &mut self.exits {
                    exits.push(at);
                }
            }
            _ => {
                self.ops.push(Op::Leave(how));
                return;
            }
        }
        self.ops.push(Op::Jump { t: LEAVE });
    }
    fn stmt(&mut self, s: &Stmt<CTy>) {
        match s {
            Stmt::Let(l, e) => self.assign(&Place::Local(*l), e),
            // Optimized IR has none; a fact is for the facts, and the reference interpreter's.
            Stmt::Fact(_) => {}
            Stmt::Assign(p, e) => self.assign(p, e),
            Stmt::Expr(e) => {
                let n = self.size(&e.ty);
                let d = self.temp(n);
                self.expr(e, d);
            }
            Stmt::If(c, a, b) => {
                let c = self.operand(c);
                let skip = self.ops.len();
                self.ops.push(Op::JumpUnless { c, t: LEAVE });
                self.block(a);
                if b.is_empty() {
                    let t = self.here();
                    self.patch(skip, t);
                } else {
                    let end = self.ops.len();
                    self.ops.push(Op::Jump { t: LEAVE });
                    let t = self.here();
                    self.patch(skip, t);
                    self.block(b);
                    let t = self.here();
                    self.patch(end, t);
                }
            }
            Stmt::Loop(body, step, _) => {
                self.looping += 1;
                let head = self.here();
                self.loops.push(Ctx::Loop {
                    brk: Vec::new(),
                    cont: Vec::new(),
                });
                self.block(body);
                let Some(Ctx::Loop { brk, cont }) = self.loops.pop() else {
                    return self.fail("the engine lost a loop's context");
                };
                let at_step = self.here();
                for j in cont {
                    self.patch(j, at_step);
                }
                self.loops.push(Ctx::Step { end: Vec::new() });
                self.block(step);
                let Some(Ctx::Step { end }) = self.loops.pop() else {
                    return self.fail("the engine lost a step's context");
                };
                let back = self.here();
                for j in end {
                    self.patch(j, back);
                }
                self.ops.push(Op::Jump { t: head });
                let out = self.here();
                for j in brk {
                    self.patch(j, out);
                }
                self.looping -= 1;
            }
            Stmt::Switch(c, arms) => {
                let c = self.operand(c);
                let table = self.c.tables.len() as u32;
                self.c.tables.push(Vec::new());
                self.ops.push(Op::Switch { c, table });
                let mut ends = Vec::new();
                for arm in arms {
                    let t = self.here();
                    self.c.tables[table as usize].push(t);
                    self.block(arm);
                    ends.push(self.ops.len());
                    self.ops.push(Op::Jump { t: LEAVE });
                }
                let out = self.here();
                for j in ends {
                    self.patch(j, out);
                }
            }
            Stmt::Break => self.leave(Leave::Break),
            Stmt::Continue => self.leave(Leave::Continue),
            Stmt::Return(e) => {
                if let Some(e) = e {
                    let n = self.size(&e.ty);
                    if n == 1 && !self.local_at(self.ret) {
                        let ret = self.ret;
                        self.expr(e, ret);
                        self.leave(Leave::Return);
                        return;
                    }
                    let d = self.temp(n);
                    self.expr(e, d);
                    if n > 0 {
                        self.ops.push(Op::Move {
                            d: self.ret,
                            s: d,
                            n,
                        });
                    }
                }
                self.leave(Leave::Return);
            }
            Stmt::Check {
                kind,
                cond,
                msg,
                site,
            } => {
                let c = self.operand(cond);
                let m = self.c.msgs.len() as u32;
                self.c.msgs.push(msg.clone());
                self.ops.push(Op::Check {
                    c,
                    kind: *kind,
                    origin: self.origin(*site),
                    msg: m,
                });
            }
            Stmt::Event(ev) => {
                let busy = self.temp(1);
                self.ops.push(Op::Showing { d: busy });
                let skip = self.ops.len();
                self.ops.push(Op::JumpIf { c: busy, t: 0 });
                for v in &ev.vals {
                    self.ops.push(Op::Capture);
                    let n = self.size(&v.ty);
                    let d = self.temp(n);
                    self.expr(v, d);
                }
                let e = self.c.events.len() as u32;
                self.c.events.push(EventCode {
                    kind: ev.kind.clone(),
                    func: ev.func,
                    site: ev.site,
                    vals: ev.vals.len(),
                });
                self.ops.push(Op::Event { e });
                let end = self.here();
                self.patch(skip, end);
            }
            Stmt::Region {
                target,
                args,
                var,
                lend,
                body,
                site,
                ..
            } => {
                let Target::Inst(f) = target else {
                    self.fail("an unresolved region call reached the engine");
                    return;
                };
                if *lend && self.inlinable_with_body(*f, body) {
                    let var = self.off[*var];
                    self.inline(*f, args, self.origin(*site), 0, Some((var, body.clone())));
                    return;
                }
                let c = self.call(*f, args, self.origin(*site), 0);
                let block = self.region_body(body);
                let at = self.ops.len();
                self.ops.push(Op::Region {
                    c,
                    body: block,
                    var: self.off[*var],
                    lend: *lend,
                    jumps: Jumps {
                        brk: LEAVE,
                        cont: LEAVE,
                        other: LEAVE,
                    },
                });
                self.pads(at);
            }
            Stmt::Yield(e) => {
                let n = self.size(&e.ty);
                let s = self.temp(n);
                self.expr(e, s);
                let lay = self.lay(&e.ty);
                self.ops.push(Op::Yield { s, lay });
            }
            Stmt::Drop(p) => {
                let ty = self.place_ty(p);
                let (n, lay) = (self.size(&ty), self.lay(&ty));
                let (s, at) = match self.direct(p) {
                    Some(s) => (s, None),
                    None => {
                        let d = self.temp(n);
                        let p = self.place(p);
                        self.ops.push(Op::Take { d, p, n });
                        (d, Some(p))
                    }
                };
                self.ops.push(Op::Drop { s, lay });
                self.ops.push(Op::Zero { d: s, n });
                if let Some(p) = at {
                    self.ops.push(Op::Put { p, s, n });
                }
            }
            Stmt::LendOut(p) if self.lending.is_some() => self.hand_out_in_place(p),
            Stmt::LendOut(p) => {
                let ty = self.place_ty(p);
                let lay = self.lay(&ty);
                let p = self.place(p);
                self.ops.push(Op::LendOut { p, lay });
            }
        }
    }
    /// The flow a region call hands back goes where the same flow written at the call
    /// would: pads after the op hold a `break`, a `continue` and, in a step, the step's end.
    fn pads(&mut self, at: usize) {
        if self.loops.is_empty() {
            return;
        }
        let over = self.ops.len();
        self.ops.push(Op::Jump { t: LEAVE });
        let brk = self.here();
        self.leave(Leave::Break);
        let cont = self.here();
        self.leave(Leave::Continue);
        let other = match self.loops.last() {
            Some(Ctx::Step { .. }) => {
                let t = self.here();
                self.leave(Leave::Return);
                t
            }
            _ => LEAVE,
        };
        let t = self.here();
        self.patch(over, t);
        if let Op::Region { jumps, .. } = &mut self.ops[at] {
            *jumps = Jumps { brk, cont, other };
        }
    }
    /// A region body: a block of its own, run in this frame. Its end is a normal end.
    fn region_body(&mut self, body: &[Stmt<CTy>]) -> u32 {
        let ops = std::mem::take(&mut self.ops);
        let marks = std::mem::replace(&mut self.marks, vec![(0, self.chain)]);
        let loops = std::mem::take(&mut self.loops);
        self.block(body);
        let body_ops = std::mem::replace(&mut self.ops, ops);
        let body_marks = std::mem::replace(&mut self.marks, marks);
        self.loops = loops;
        let b = self.c.blocks.len() as u32;
        self.c.blocks.push(body_ops);
        self.c.marks.push(body_marks);
        b
    }
    fn text(&mut self, t: &str) -> u32 {
        let i = self.c.texts.len() as u32;
        self.c.texts.push(t.to_string());
        i
    }
}

/// Whether evaluating `e` can run a call, which may write a local through a reference.
fn calls(e: &Expr<CTy>) -> bool {
    match &e.kind {
        EK::Call(..) | EK::CallValue(..) => true,
        EK::Intr(_, args, _) => args.iter().any(|a| matches!(a, Arg::Value(v) if calls(v))),
        EK::MakeStruct(xs) | EK::MakeClosure(xs) => xs.iter().any(calls),
        EK::Inject(x) | EK::Widen(x) | EK::MakeUnion(_, x) => calls(x),
        _ => false,
    }
}

impl Fc<'_> {
    fn assign(&mut self, p: &Place, e: &Expr<CTy>) {
        let ty = self.place_ty(p);
        let n = self.size(&ty);
        // The value goes straight into a local when the op writing it reads everything it
        // needs first. Ownership released what the place held before (`Stmt::Drop`).
        if let Some(d) = self.direct(p) {
            let direct = matches!(
                e.kind,
                EK::Int(_)
                    | EK::F64(_)
                    | EK::Bool(_)
                    | EK::Unit
                    | EK::Copy(_)
                    | EK::Intr(..)
                    | EK::Call(..)
            );
            if direct {
                self.expr(e, d);
            } else {
                let s = self.temp(n);
                self.expr(e, s);
                if n > 0 {
                    self.ops.push(Op::Move { d, s, n });
                }
            }
            return;
        }
        let s = self.temp(n);
        self.expr(e, s);
        match self.global(p) {
            Some(a) => self.ops.push(Op::PutG { a, s, n }),
            None => {
                let p = self.place(p);
                self.ops.push(Op::Put { p, s, n });
            }
        }
    }
    /// The register the fn's entry puts `v` in, while registers for constants are left; a
    /// region body runs in its fn's frame, so it reads them too.
    fn konst(&mut self, v: u64) -> Option<Reg> {
        let (base, k) = self.kregs;
        let j = match self.consts.iter().position(|&c| c == v) {
            Some(j) => j,
            None if self.consts.len() < k as usize => {
                self.consts.push(v);
                self.consts.len() - 1
            }
            None => return None,
        };
        Some(base + j as Reg)
    }
    /// The register holding `e`'s value: a local's own when reading it copies nothing.
    /// Storage of characters into a fresh register, and whether the op owns it then: `None` for
    /// lent storage, which text never takes.
    fn text_arg(&mut self, r: &Arg<CTy>) -> Option<(Reg, bool)> {
        let t = self.temp(1);
        let owned = match r {
            Arg::Value(r) => {
                self.expr(r, t);
                true
            }
            Arg::View(p) => {
                let p = self.place(p);
                self.ops.push(Op::Peek { d: t, p });
                false
            }
            Arg::Lend(_) => return None,
        };
        Some((t, owned))
    }
    fn operand(&mut self, e: &Expr<CTy>) -> Reg {
        if let (EK::Int(i), true) = (&e.kind, self.looping > 0) {
            if let Some(r) = self.konst(*i as u64) {
                return r;
            }
        }
        if let EK::Copy(p) = &e.kind {
            if let Some(r) = self.direct(p) {
                self.retain(r, &e.ty);
                return r;
            }
        }
        let n = self.size(&e.ty);
        let d = self.temp(n);
        self.expr(e, d);
        d
    }
    /// A copy of a value that holds storage counts one more holder of what it holds.
    fn retain(&mut self, s: Reg, t: &CTy) {
        if self.p.stores(t) {
            let lay = self.lay(t);
            self.ops.push(Op::Retain { s, lay });
        }
    }
    /// Operands in order; one a later operand's call could write is copied first.
    fn operands(&mut self, args: &[Arg<CTy>]) -> Option<Vec<Reg>> {
        let mut regs = Vec::with_capacity(args.len());
        for (i, a) in args.iter().enumerate() {
            let Arg::Value(v) = a else {
                return None;
            };
            let later = args[i + 1..]
                .iter()
                .any(|a| matches!(a, Arg::Value(v) if calls(v)));
            let r = self.operand(v);
            if later && matches!(v.kind, EK::Copy(_)) {
                let n = self.size(&v.ty);
                let d = self.temp(n);
                self.ops.push(Op::Move { d, s: r, n });
                regs.push(d);
            } else {
                regs.push(r);
            }
        }
        Some(regs)
    }
    fn expr(&mut self, e: &Expr<CTy>, d: Reg) {
        match &e.kind {
            EK::Int(i) => self.ops.push(Op::Imm { d, v: *i as u64 }),
            EK::F64(b) => self.ops.push(Op::Imm { d, v: *b }),
            EK::Bool(b) => self.ops.push(Op::Imm {
                d,
                v: u64::from(*b),
            }),
            EK::Unit => {}
            EK::Text(t) => {
                let t = self.text(t);
                self.ops.push(Op::Text { d, t });
            }
            EK::Copy(p) | EK::Lent(p) => {
                let (n, lay) = (self.size(&e.ty), self.lay(&e.ty));
                match self.direct(p) {
                    Some(s) => {
                        if n > 0 && s != d {
                            self.ops.push(Op::Move { d, s, n });
                        }
                    }
                    None => match self.global(p) {
                        Some(a) => self.ops.push(Op::GetG { d, a, n }),
                        None => {
                            let p = self.place(p);
                            self.ops.push(Op::Load { d, p, lay });
                        }
                    },
                }
                if let EK::Copy(_) = e.kind {
                    self.retain(d, &e.ty);
                }
            }
            EK::AddrOf(Place::Local(l)) => self.ops.push(Op::Addr { d, s: self.off[*l] }),
            EK::AddrOf(Place::Global(g)) => self.ops.push(Op::Imm {
                d,
                v: u64::from(self.global_at[*g]),
            }),
            EK::AddrOf(_) => self.fail("a reference to part of a variable"),
            EK::Tag(p) => {
                let p = self.place(p);
                self.ops.push(Op::Peek { d, p });
            }
            EK::MakeStruct(xs) | EK::MakeClosure(xs) => {
                let lay = self.lay(&e.ty);
                for (i, x) in xs.iter().enumerate() {
                    let Some((at, _)) = self.c.lays.part(lay, i) else {
                        self.fail("a struct value with more fields than its type");
                        return;
                    };
                    self.expr(x, d + at);
                }
            }
            EK::MakeUnion(i, x) => {
                self.ops.push(Op::Imm { d, v: *i as u64 });
                self.expr(x, d + 1);
            }
            EK::Inject(x) | EK::Widen(x) => self.expr(x, d),
            EK::DomLen(_) | EK::DomLo(_) | EK::DomHeld(_) | EK::DomFromCount(_) => {
                self.fail("a domain value the link step did not resolve")
            }
            EK::HasBound(..) => self.fail("a bound test the link step did not decide"),
            EK::Call(Target::Inst(f), args, site) if let Some(op) = self.funcs[*f].fast => {
                self.intr(op, args, self.origin(*site), &e.ty, d)
            }
            EK::Call(Target::Inst(f), args, site) if self.inlinable(*f) => {
                self.inline(*f, args, self.origin(*site), d, None)
            }
            EK::Call(Target::Inst(f), args, site) => {
                // The call's origin runs while its arguments are evaluated, as in the reference
                // interpreter, and is the origin again after the call.
                let save = self.temp(1);
                let origin = self.origin(*site);
                self.ops.push(Op::Enter { origin, save });
                let c = self.call(*f, args, origin, d);
                self.ops.push(Op::Call { c });
            }
            EK::Call(..) => self.fail("an unresolved call reached the engine"),
            EK::CallValue(..) => self.fail("a call through a function value in canonical IR"),
            EK::Pass(..) => self.fail("a place passed before ownership decided how"),
            EK::Intr(op, args, site) => self.intr(*op, args, self.origin(*site), &e.ty, d),
        }
    }
    /// The origin of the user's file `site` is on; 0 for none.
    fn origin(&self, site: Site) -> u32 {
        self.f.origins[site as usize]
    }
    fn call(&mut self, f: usize, args: &[Arg<CTy>], origin: u32, d: Reg) -> u32 {
        let mut codes = Vec::with_capacity(args.len());
        for a in args {
            codes.push(match a {
                Arg::Value(v) => {
                    let n = self.size(&v.ty);
                    let s = self.temp(n);
                    self.expr(v, s);
                    ArgCode::Move { s, n }
                }
                // Nothing writes a viewed place during the call: the callee gets its value.
                Arg::View(p) => {
                    let ty = self.place_ty(p);
                    let (n, lay) = (self.size(&ty), self.lay(&ty));
                    let s = self.temp(n);
                    let p = self.place(p);
                    self.ops.push(Op::Load { d: s, p, lay });
                    ArgCode::Move { s, n }
                }
                Arg::Lend(p) => ArgCode::Lend { p: self.place(p) },
            });
        }
        let c = self.c.calls.len() as u32;
        self.c.calls.push(CallCode {
            f: f as u32,
            origin,
            args: codes,
            d,
        });
        c
    }
    fn intr(&mut self, op: Intr, args: &[Arg<CTy>], origin: u32, ty: &CTy, d: Reg) {
        if op.checked() && origin != 0 {
            self.ops.push(Op::At { origin });
        }
        let first_ty = match args.first() {
            Some(Arg::Value(v)) => v.ty.unlent().clone(),
            Some(Arg::View(p) | Arg::Lend(p)) => self.place_ty(p).unlent().clone(),
            None => CTy::Unit,
        };
        let elem = |fc: &mut Self, t: &CTy| match t.unlent() {
            CTy::Raw(e) => Some(fc.lay(e)),
            _ => None,
        };
        if ops::on_scalars(op) {
            return self.scalar(op, args, d, &first_ty);
        }
        match op {
            Intr::CheckRange
            | Intr::CheckKey
            | Intr::CheckVariant
            | Intr::CheckToken
            | Intr::CheckInput => {
                let Some(&[c]) = self.operands(args).as_deref() else {
                    return self.fail("a lent operand");
                };
                self.ops.push(Op::Guard { op, c });
            }
            Intr::RawEmpty => match elem(self, ty) {
                Some(elem) => self.ops.push(Op::RawEmpty { d, elem }),
                None => self.fail("storage of a type that is not storage"),
            },
            Intr::RawNew => {
                let (Some(Arg::Value(n)), Some(Arg::Value(fill)), Some(elem)) =
                    (args.first(), args.get(1), elem(self, ty))
                else {
                    return self.fail("lent fill");
                };
                let n = self.operand(n);
                let size = self.size(&fill.ty);
                let f = self.temp(size);
                self.expr(fill, f);
                self.ops.push(Op::RawNew {
                    d,
                    n,
                    fill: f,
                    elem,
                });
            }
            Intr::Flush => self.ops.push(Op::Flush),
            Intr::Out => {
                let (Some(r), Some(Arg::Value(n))) = (args.first(), args.get(1)) else {
                    return self.fail("output without storage and a count");
                };
                let Some((t, owned)) = self.text_arg(r) else {
                    return self.fail("lent storage in an output");
                };
                let n = self.operand(n);
                self.ops.push(Op::Out { r: t, n, owned });
            }
            Intr::RawGet => {
                let (Some(r), Some(Arg::Value(i)), Some(elem)) =
                    (args.first(), args.get(1), elem(self, &first_ty))
                else {
                    return self.fail("read of non-storage");
                };
                let t = self.temp(1);
                let owned = match r {
                    Arg::Value(r) => {
                        self.expr(r, t);
                        true
                    }
                    Arg::View(p) => {
                        let p = self.place(p);
                        self.ops.push(Op::Peek { d: t, p });
                        false
                    }
                    Arg::Lend(_) => return self.fail("lent storage in a read"),
                };
                let i = self.operand(i);
                self.ops.push(Op::RawGet {
                    d,
                    r: t,
                    i,
                    elem,
                    owned,
                });
            }
            Intr::RawSet => {
                let (Some(Arg::Lend(p)), Some(Arg::Value(i)), Some(Arg::Value(x)), Some(elem)) = (
                    args.first(),
                    args.get(1),
                    args.get(2),
                    elem(self, &first_ty),
                ) else {
                    return self.fail("storage written without a lend");
                };
                let i = self.operand(i);
                let size = self.size(&x.ty);
                let xr = self.temp(size);
                self.expr(x, xr);
                let p = self.place(p);
                self.ops.push(Op::RawSet { p, i, x: xr, elem });
            }
            Intr::RawTake => {
                let (Some(Arg::Lend(p)), Some(Arg::Value(i)), Some(elem)) =
                    (args.first(), args.get(1), elem(self, &first_ty))
                else {
                    return self.fail("storage moved from without a lend");
                };
                let i = self.operand(i);
                let p = self.place(p);
                self.ops.push(Op::RawTake { d, p, i, elem });
            }
            Intr::RawCap => match (args.first(), elem(self, ty)) {
                (Some(Arg::Value(n)), Some(elem)) => {
                    let n = self.operand(n);
                    self.ops.push(Op::RawCap { d, n, elem });
                }
                _ => self.fail("storage of a type that is not storage"),
            },
            Intr::InputLeft => self.ops.push(Op::InputLeft { d }),
            Intr::ReadInt => self.ops.push(Op::ReadInt { d }),
            Intr::ReadByte => self.ops.push(Op::ReadByte { d }),
            Intr::ParseF64 => {
                let (Some(r), Some(Arg::Value(n))) = (args.first(), args.get(1)) else {
                    return self.fail("a number parse without storage and a count");
                };
                let Some((t, owned)) = self.text_arg(r) else {
                    return self.fail("lent storage in a number parse");
                };
                let n = self.operand(n);
                self.ops.push(Op::ParseF64 { d, r: t, n, owned });
            }
            Intr::PeekByte => {
                let Some(&[k]) = self.operands(args).as_deref() else {
                    return self.fail("a lent operand");
                };
                self.ops.push(Op::PeekByte { d, k });
            }
            Intr::RandWord => self.ops.push(Op::RandWord { d }),
            // A value never read still holds no handle, so copying or dropping its holder is sound.
            Intr::Unset => {
                let n = self.size(ty);
                self.ops.push(Op::Zero { d, n });
            }
            _ => self.fail("a scalar operation outside `ops::on_scalars`"),
        }
    }
    /// An operation on scalars: `ops` gives what it computes; this picks only how its
    /// registers are read.
    fn scalar(&mut self, op: Intr, args: &[Arg<CTy>], d: Reg, first_ty: &CTy) {
        let sig = ops::sig(op, ops::Operand::of(first_ty));
        let Some(rs) = self.operands(args) else {
            return self.fail("a lent operand");
        };
        let wide = sig.args[0] == ops::Operand::I128;
        let code = match *rs.as_slice() {
            [a] => Op::Unary { op, sig, d, a },
            [a, b] => {
                let kind = kind(first_ty);
                match (ops::compares(op), kind) {
                    (true, _) if wide => Op::WideCmp { op, d, a, b },
                    (false, _) if wide => Op::Wide {
                        op,
                        d,
                        a,
                        b,
                        narrow: sig.args[1].words() == 1,
                    },
                    (true, Kind::Int | Kind::Bool | Kind::U64 | Kind::F64) => {
                        Op::Cmp { op, d, a, b, kind }
                    }
                    (true, _) => Op::Fail {
                        rule: "comparison of a value of another kind",
                    },
                    (false, Kind::U64) => Op::Word { op, d, a, b },
                    (false, Kind::F64) => Op::Float { op, d, a, b },
                    (false, _) => Op::Int { op, d, a, b },
                }
            }
            _ => Op::Fail {
                rule: "a scalar operation of another arity",
            },
        };
        self.ops.push(code);
    }
    /// A place's root, its steps and its type.
    fn walk(&mut self, p: &Place) -> (Root, Vec<Step>, CTy) {
        let off = |root: &mut Root, steps: &mut Vec<Step>, at: u32| match (steps.last_mut(), root) {
            (Some(Step::Off(x)), _) => *x += at,
            (Some(_), _) => steps.push(Step::Off(at)),
            (None, Root::Local(r)) => *r += at,
            (None, Root::Global(g)) => *g += at,
        };
        match p {
            Place::Local(l) => (
                Root::Local(self.off[*l]),
                Vec::new(),
                self.f.locals[*l].clone(),
            ),
            Place::Global(g) => (
                Root::Global(self.global_at[*g]),
                Vec::new(),
                self.p.globals[*g].clone(),
            ),
            Place::Field(b, i) | Place::Variant(b, i) => {
                let (mut root, mut steps, t) = self.walk(b);
                let t = t.unlent().clone();
                let lay = self.lay(&t);
                let at = self.c.lays.part(lay, *i).map_or(0, |(at, _)| at);
                off(&mut root, &mut steps, at);
                let part =
                    (self.c.lays.part(lay, *i)).map(|(_, l)| self.c.lays.ty[l as usize].clone());
                (root, steps, part.unwrap_or(CTy::Error))
            }
            Place::Deref(b) => {
                let (root, mut steps, t) = self.walk(b);
                steps.push(Step::Deref);
                let t = match t.unlent() {
                    CTy::Ref(t) => (**t).clone(),
                    _ => CTy::Error,
                };
                (root, steps, t)
            }
            Place::Through(b) => {
                let (root, steps, t) = self.walk(b);
                (root, steps, t.unlent().clone())
            }
            Place::Slot(b, l) => {
                let (root, mut steps, t) = self.walk(b);
                let t = match t.unlent() {
                    CTy::Raw(e) => (**e).clone(),
                    _ => CTy::Error,
                };
                let stride = self.size(&t);
                steps.push(Step::Slot {
                    i: self.off[*l],
                    stride,
                });
                (root, steps, t)
            }
        }
    }
    /// The register a place is when no step reaches it: a local or a part of one.
    fn direct(&mut self, p: &Place) -> Option<Reg> {
        match self.walk(p) {
            (Root::Local(r), steps, _) if steps.is_empty() => Some(r),
            _ => None,
        }
    }
    /// Where a lent place's words are: registers, a global's words, or a place to resolve.
    fn spot(&mut self, p: &Place) -> Spot {
        match self.walk(p) {
            (Root::Local(r), steps, _) if steps.is_empty() => Spot::Reg(r),
            (Root::Global(a), steps, _) if steps.is_empty() => Spot::Global(a),
            _ => Spot::Place(self.place(p)),
        }
    }
    fn take(&mut self, d: Reg, at: Spot, n: u32) {
        match at {
            Spot::Reg(s) if n > 0 && s != d => self.ops.push(Op::Move { d, s, n }),
            Spot::Reg(_) => {}
            Spot::Global(a) => self.ops.push(Op::GetG { d, a, n }),
            Spot::Place(p) => self.ops.push(Op::Take { d, p, n }),
        }
    }
    fn put(&mut self, at: Spot, s: Reg, n: u32) {
        match at {
            Spot::Reg(d) if n > 0 && s != d => self.ops.push(Op::Move { d, s, n }),
            Spot::Reg(_) => {}
            Spot::Global(a) => self.ops.push(Op::PutG { a, s, n }),
            Spot::Place(p) => self.ops.push(Op::Put { p, s, n }),
        }
    }
    /// The stack address of a place no step reaches from a global.
    fn global(&mut self, p: &Place) -> Option<u32> {
        match self.walk(p) {
            (Root::Global(a), steps, _) if steps.is_empty() => Some(a),
            _ => None,
        }
    }
    fn place_ty(&mut self, p: &Place) -> CTy {
        self.walk(p).2
    }
    fn place(&mut self, p: &Place) -> u32 {
        let (root, steps, _) = self.walk(p);
        let i = self.c.places.len() as u32;
        self.c.places.push(PlaceCode { root, steps });
        i
    }
}

/// How much code a fn body is, to bound what inlining copies; a fn that hands code to its
/// caller's region is never inlined.
fn cost(b: &[Stmt<CTy>]) -> usize {
    fn e(x: &Expr<CTy>) -> usize {
        1 + match &x.kind {
            EK::Intr(_, args, _) | EK::Call(_, args, _) => args
                .iter()
                .map(|a| match a {
                    Arg::Value(v) => e(v),
                    _ => 1,
                })
                .sum(),
            EK::CallValue(f, args, _) => {
                let arg = |a: &Arg<CTy>| match a {
                    Arg::Value(v) => e(v),
                    _ => 1,
                };
                e(f) + args.iter().map(arg).sum::<usize>()
            }
            EK::MakeStruct(xs) | EK::MakeClosure(xs) => xs.iter().map(e).sum(),
            EK::Inject(x) | EK::Widen(x) | EK::MakeUnion(_, x) => e(x),
            _ => 0,
        }
    }
    b.iter()
        .map(|s| {
            1 + match s {
                Stmt::Let(_, x) | Stmt::Assign(_, x) | Stmt::Expr(x) => e(x),
                Stmt::Fact(_) => 0,
                Stmt::Return(x) => x.as_ref().map_or(0, e),
                Stmt::If(c, a, b) => e(c) + cost(a) + cost(b),
                Stmt::Loop(a, b, _) => cost(a) + cost(b),
                Stmt::Switch(c, arms) => e(c) + arms.iter().map(|a| cost(a)).sum::<usize>(),
                Stmt::Check { cond, .. } => e(cond),
                Stmt::Event(cpl_next_ir::ir::EventPoint { vals: vs, .. }) => vs.iter().map(e).sum(),
                Stmt::Region { .. } | Stmt::Yield(_) => INLINE_COST,
                Stmt::LendOut(_) | Stmt::Drop(_) => 0,
                Stmt::Break | Stmt::Continue => 0,
            }
        })
        .sum()
}
const INLINE_COST: usize = 40;

impl<'a> Fc<'a> {
    fn inlinable(&self, f: usize) -> bool {
        let g = &self.funcs[f];
        g.region.is_none()
            && f != self.c.main as usize
            && !self.inlining.contains(&f)
            && self.inlining.len() < 4
            && cost(&g.body) < INLINE_COST
    }
    /// A lender inlined with the region body its caller hands it: one `lend`, which no flow
    /// of the body leaves.
    fn inlinable_with_body(&self, f: usize, body: &[Stmt<CTy>]) -> bool {
        let g = &self.funcs[f];
        matches!(g.region, Some((true, _)))
            && f != self.c.main as usize
            && !self.inlining.contains(&f)
            && self.inlining.len() < 4
            && cost(&g.body) < INLINE_COST
            && hand_outs(&g.body) == 1
            && !leaves(body, 0)
    }
    /// An inlined lender's `lend`: the place moves to the region's variable, the caller's body
    /// runs in the caller's context, and the variable moves back, as `Op::LendOut` does.
    fn hand_out_in_place(&mut self, p: &Place) {
        let Some(mut l) = self.lending.take() else {
            return;
        };
        let ty = self.place_ty(p);
        let n = self.size(&ty);
        let at = self.spot(p);
        self.take(l.var, at, n);
        let save = self.temp(1);
        self.ops.push(Op::Enter { origin: 0, save });
        let off = std::mem::replace(&mut self.off, std::mem::take(&mut l.off));
        let f = std::mem::replace(&mut self.f, l.f);
        let chain = std::mem::replace(&mut self.chain, l.chain);
        let loops = std::mem::take(&mut self.loops);
        let exits = self.exits.take();
        self.lending = l.outer.take().map(|b| *b);
        self.marks.push((self.ops.len() as u32, self.chain));
        let body = std::mem::take(&mut l.body);
        self.block(&body);
        self.marks.push((self.ops.len() as u32, chain));
        l.outer = self.lending.take().map(Box::new);
        l.off = std::mem::replace(&mut self.off, off);
        self.f = f;
        self.chain = chain;
        self.loops = loops;
        self.exits = exits;
        self.ops.push(Op::Restore { save });
        self.put(at, l.var, n);
        self.ops.push(Op::Zero { d: l.var, n });
        self.lending = Some(l);
    }
    /// A call compiled into the caller: the callee's locals are registers of this frame, its
    /// `return` a jump to the end, and its origins, events and drops those of the call. A
    /// lender gets the region body it lends to (`hand_out_in_place`).
    fn inline(
        &mut self,
        f: usize,
        args: &[Arg<CTy>],
        origin: u32,
        d: Reg,
        lend: Option<(Reg, Vec<Stmt<CTy>>)>,
    ) {
        let lent_to = self.lending.take();
        let (lending, lent_to) = match lend {
            Some((var, body)) => (
                Some(Lending {
                    var,
                    body,
                    off: self.off.clone(),
                    f: self.f,
                    chain: self.chain,
                    outer: lent_to.map(Box::new),
                }),
                None,
            ),
            None => (None, lent_to),
        };
        let save = self.temp(1);
        self.ops.push(Op::Enter { origin, save });
        let funcs: &'a [Func<CTy>] = self.funcs;
        let g = &funcs[f];
        let base = self.temp(self.c.funcs[f].locals);
        let mut off: Vec<Reg> = self.offs[f].iter().map(|o| base + o).collect();
        let drops: Vec<(Reg, Lay)> = self.c.funcs[f]
            .drops
            .iter()
            .map(|&(at, l)| (base + at, l))
            .collect();
        for &(at, lay) in &drops {
            let n = self.c.lays.size[lay as usize];
            self.ops.push(Op::Zero { d: at, n });
        }
        // A parameter the body never writes, given a local of the caller that nothing writes
        // while the call runs, is that local's registers: no words move in.
        let lent_body = lending.as_ref().map(|l| &l.body[..]);
        let mut aliased = Vec::new();
        for ((&(l, _), &view), a) in g.params.iter().zip(&g.views).zip(args) {
            let x = match a {
                // A viewed place stays as it is while the call runs.
                Arg::View(Place::Local(x)) if view && !addressed(&g.body, l) => *x,
                Arg::Value(Expr {
                    kind: EK::Copy(Place::Local(x)),
                    ..
                }) if !writes(&g.body, l) && !drops.iter().any(|&(at, _)| at == off[l]) => *x,
                _ => continue,
            };
            if writes_args(args, x)
                || addressed(&self.f.body, x)
                || lent_body.is_some_and(|b| writes(b, x))
            {
                continue;
            }
            if let Some(s) = self.direct(&Place::Local(x)) {
                off[l] = s;
                aliased.push(l);
            }
        }
        let mut lends = Vec::new();
        for (&(l, _), a) in g.params.iter().zip(args) {
            let to = off[l];
            if aliased.contains(&l) {
                continue;
            }
            match a {
                Arg::Value(v) => self.expr(v, to),
                Arg::View(p) => {
                    let ty = self.place_ty(p);
                    let lay = self.lay(&ty);
                    let p = self.place(p);
                    self.ops.push(Op::Load { d: to, p, lay });
                }
                Arg::Lend(p) => lends.push((p, to)),
            }
        }
        let mut back = Vec::with_capacity(lends.len());
        for (p, to) in lends {
            let ty = self.place_ty(p);
            let n = self.size(&ty);
            let p = self.spot(p);
            self.take(to, p, n);
            back.push((p, to, n));
        }
        let off = std::mem::replace(&mut self.off, off);
        let caller = std::mem::replace(&mut self.f, g);
        let ret = std::mem::replace(&mut self.ret, d);
        let exits = self.exits.replace(Vec::new());
        let loops = std::mem::take(&mut self.loops);
        let outer = self.chain;
        let mut chain = self.c.chains[outer as usize].clone();
        chain.push((f as u32, save));
        self.chain = self.c.chains.len() as u32;
        self.c.chains.push(chain);
        self.marks.push((self.ops.len() as u32, self.chain));
        self.inlining.push(f);
        self.lending = lending;
        self.block(&g.body);
        let lending = self.lending.take();
        self.inlining.pop();
        self.chain = outer;
        self.marks.push((self.ops.len() as u32, outer));
        self.loops = loops;
        let mine = std::mem::replace(&mut self.exits, exits);
        self.ret = ret;
        self.f = caller;
        self.off = off;
        let end = self.here();
        for j in mine.unwrap_or_default() {
            self.patch(j, end);
        }
        for (s, lay) in drops {
            self.ops.push(Op::Drop { s, lay });
        }
        for (p, s, n) in back {
            self.put(p, s, n);
        }
        self.ops.push(Op::Restore { save });
        self.lending = lending.and_then(|l| l.outer).map(|b| *b).or(lent_to);
    }
}

/// How many `lend`s a lender's body runs at most once each; one in a nested region counts
/// as many.
fn hand_outs(b: &[Stmt<CTy>]) -> usize {
    b.iter()
        .map(|s| match s {
            Stmt::LendOut(_) => 1,
            Stmt::If(_, x, y) | Stmt::Loop(x, y, _) => hand_outs(x) + hand_outs(y),
            Stmt::Switch(_, arms) => arms.iter().map(|a| hand_outs(a)).sum(),
            Stmt::Region { body, .. } if hand_outs(body) > 0 => 2,
            _ => 0,
        })
        .sum()
}
/// Whether a flow of `b` leaves it: a `return`, or a `break` or `continue` of no loop in it.
fn leaves(b: &[Stmt<CTy>], depth: usize) -> bool {
    b.iter().any(|s| match s {
        Stmt::Return(_) => true,
        Stmt::Break | Stmt::Continue => depth == 0,
        Stmt::If(_, x, y) => leaves(x, depth) || leaves(y, depth),
        Stmt::Loop(x, y, _) => leaves(x, depth + 1) || leaves(y, depth + 1),
        Stmt::Switch(_, arms) => arms.iter().any(|a| leaves(a, depth)),
        Stmt::Region { body, .. } => leaves(body, depth),
        _ => false,
    })
}
