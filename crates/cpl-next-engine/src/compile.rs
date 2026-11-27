//! Canonical IR to engine code: each fn becomes a block of `Op`s over word registers of its
//! frame (its result, then its locals, then temporaries); each region body is a block of its
//! own, run in the frame of the fn it is written in.
use crate::layout::{Lay, Layouts};
use cpl_next_ir::ir::*;
use cpl_next_ir::prog::{CTy, Program};

/// A word offset in the running frame.
pub(crate) type Reg = u32;
/// No jump target: the flow leaves the block.
pub(crate) const LEAVE: u32 = u32::MAX;

/// How a word prints, compares and shows in an event.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Kind {
    Int,
    Bool,
    Char,
    U64,
    Other,
}
fn kind(t: &CTy) -> Kind {
    match t {
        CTy::Int | CTy::Dom(_) => Kind::Int,
        CTy::Bool => Kind::Bool,
        CTy::Char => Kind::Char,
        CTy::U64 => Kind::U64,
        CTy::Lent(t) => kind(t),
        _ => Kind::Other,
    }
}
fn unlent(t: &CTy) -> &CTy {
    match t {
        CTy::Lent(t) => t,
        t => t,
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
    /// The place's old value is dropped and the register's moves in.
    Store {
        p: u32,
        s: Reg,
        lay: Lay,
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
    Neg {
        d: Reg,
        a: Reg,
    },
    Not {
        d: Reg,
        a: Reg,
    },
    Cmp {
        op: Intr,
        d: Reg,
        a: Reg,
        b: Reg,
        kind: Kind,
    },
    At {
        site: Site,
    },
    /// A call's line starts running: `save` keeps the line to go back to after it.
    Enter {
        site: Site,
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
    Switch {
        c: Reg,
        table: u32,
    },
    Leave(Leave),
    Check {
        c: Reg,
        kind: CheckKind,
        site: Site,
        msg: u32,
    },
    Guard {
        op: Intr,
        c: Reg,
    },
    Print {
        s: Reg,
        kind: Kind,
    },
    PrintText {
        t: u32,
    },
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
    InputLeft {
        d: Reg,
    },
    ReadInt {
        d: Reg,
    },
    ToU64 {
        d: Reg,
        a: Reg,
    },
    FromU64 {
        d: Reg,
        a: Reg,
    },
    CharOf {
        d: Reg,
        a: Reg,
    },
    Fail {
        rule: &'static str,
    },
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
    pub site: Site,
    pub args: Vec<ArgCode>,
    pub d: Reg,
    /// Where `Op::Enter` kept the line to go back to; a region call enters at the op.
    pub save: Option<Reg>,
}
pub(crate) struct FnCode {
    pub block: u32,
    pub frame: u32,
    pub ret: u32,
    /// Each parameter's register and size.
    pub params: Vec<(Reg, u32)>,
    pub lent: Vec<bool>,
    /// What the frame owns when the fn ends: its locals but the lent parameters.
    pub drops: Vec<(Reg, Lay)>,
}
pub(crate) struct EventCode {
    pub label: String,
    pub vals: Vec<(Reg, Kind, Lay)>,
}

pub(crate) struct Code {
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
    pub main: u32,
    /// The layout of a string literal's characters.
    pub char_lay: Lay,
}

pub(crate) fn compile(p: &Program, funcs: &[Func<CTy>]) -> Code {
    let mut c = Code {
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
        main: p.main as u32,
        char_lay: 0,
    };
    c.char_lay = c.lays.of(p, &CTy::Char);
    let mut global_at = Vec::with_capacity(p.globals.len());
    for g in &p.globals {
        global_at.push(c.globals);
        let l = c.lays.of(p, g);
        c.globals += c.lays.size[l as usize];
    }
    // Every fn's frame shape first: a call needs its callee's.
    let mut offs = Vec::with_capacity(funcs.len());
    for f in funcs {
        let ret = c.lays.of(p, &f.ret);
        let mut at = c.lays.size[ret as usize];
        let mut off = Vec::with_capacity(f.locals.len());
        let mut drops = Vec::new();
        for (i, t) in f.locals.iter().enumerate() {
            let l = c.lays.of(p, t);
            off.push(at);
            if c.lays.owns[l as usize] && !f.params.iter().any(|(x, lent)| *x == i && *lent) {
                drops.push((at, l));
            }
            at += c.lays.size[l as usize];
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
            drops,
        });
        offs.push(off);
    }
    for (i, f) in funcs.iter().enumerate() {
        let top = c.funcs[i].frame;
        let mut fc = Fc {
            c: &mut c,
            p,
            f,
            off: &offs[i],
            global_at: &global_at,
            top,
            max: top,
            ops: Vec::new(),
            loops: Vec::new(),
        };
        fc.block(&f.body);
        let (ops, max) = (std::mem::take(&mut fc.ops), fc.max);
        c.funcs[i].frame = max;
        c.funcs[i].block = c.blocks.len() as u32;
        c.blocks.push(ops);
    }
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
    off: &'a [Reg],
    global_at: &'a [u32],
    top: Reg,
    max: Reg,
    ops: Vec<Op>,
    loops: Vec<Ctx>,
}

impl Fc<'_> {
    fn lay(&mut self, t: &CTy) -> Lay {
        self.c.lays.of(self.p, t)
    }
    fn size(&mut self, t: &CTy) -> u32 {
        let l = self.lay(t);
        self.c.lays.size[l as usize]
    }
    fn owns(&mut self, t: &CTy) -> bool {
        let l = self.lay(t);
        self.c.lays.owns[l as usize]
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
            Op::Jump { t: x } | Op::JumpUnless { t: x, .. } => *x = t,
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
            Stmt::Assign(p, e) => self.assign(p, e),
            Stmt::Expr(e) => {
                let n = self.size(&e.ty);
                let d = self.temp(n);
                self.expr(e, d);
                if self.owns(&e.ty) {
                    let lay = self.lay(&e.ty);
                    self.ops.push(Op::Drop { s: d, lay });
                }
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
            Stmt::Loop(body, step) => {
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
                    let d = self.temp(n);
                    self.expr(e, d);
                    if n > 0 {
                        self.ops.push(Op::Move { d: 0, s: d, n });
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
                    site: *site,
                    msg: m,
                });
            }
            Stmt::Print(ps) => {
                for piece in ps {
                    match piece {
                        PrintPiece::Text(t) => {
                            let t = self.text(t);
                            self.ops.push(Op::PrintText { t });
                        }
                        PrintPiece::Value(v) => {
                            let kind = kind(&v.ty);
                            if kind == Kind::Other {
                                self.fail("print of a value of another kind");
                                continue;
                            }
                            let s = self.operand(v);
                            self.ops.push(Op::Print { s, kind });
                        }
                    }
                }
            }
            Stmt::Event(label, vals) => {
                let mut regs = Vec::with_capacity(vals.len());
                for v in vals {
                    let n = self.size(&v.ty);
                    let d = self.temp(n);
                    self.expr(v, d);
                    let lay = self.lay(&v.ty);
                    regs.push((d, kind(&v.ty), lay));
                }
                let e = self.c.events.len() as u32;
                self.c.events.push(EventCode {
                    label: label.clone(),
                    vals: regs,
                });
                self.ops.push(Op::Event { e });
            }
            Stmt::Region {
                target,
                args,
                var,
                lend,
                body,
                site,
            } => {
                let Target::Inst(f) = target else {
                    self.fail("an unresolved region call reached the engine");
                    return;
                };
                let c = self.call(*f, args, *site, 0, None);
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
        let loops = std::mem::take(&mut self.loops);
        self.block(body);
        let body_ops = std::mem::replace(&mut self.ops, ops);
        self.loops = loops;
        let b = self.c.blocks.len() as u32;
        self.c.blocks.push(body_ops);
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
        let (n, owns, lay) = (self.size(&ty), self.owns(&ty), self.lay(&ty));
        // A value with no storage goes straight into a local when the op writing it reads
        // everything it needs first.
        if let (Place::Local(l), false) = (p, owns) {
            let direct = matches!(
                e.kind,
                EK::Int(_) | EK::Bool(_) | EK::Unit | EK::Copy(_) | EK::Intr(..) | EK::Call(..)
            );
            let d = self.off[*l];
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
        let p = self.place(p);
        self.ops.push(Op::Store { p, s, lay });
    }
    /// The register holding `e`'s value: a local's own when reading it copies nothing.
    fn operand(&mut self, e: &Expr<CTy>) -> Reg {
        if let EK::Copy(Place::Local(l)) = &e.kind {
            if !self.owns(&e.ty) {
                return self.off[*l];
            }
        }
        let n = self.size(&e.ty);
        let d = self.temp(n);
        self.expr(e, d);
        d
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
            if later && matches!(v.kind, EK::Copy(Place::Local(_))) {
                let d = self.temp(1);
                self.ops.push(Op::Move { d, s: r, n: 1 });
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
                let (n, owns, lay) = (self.size(&e.ty), self.owns(&e.ty), self.lay(&e.ty));
                match p {
                    Place::Local(l) if !owns => {
                        if n > 0 && self.off[*l] != d {
                            self.ops.push(Op::Move {
                                d,
                                s: self.off[*l],
                                n,
                            });
                        }
                    }
                    _ => {
                        let p = self.place(p);
                        self.ops.push(Op::Load { d, p, lay });
                    }
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
            EK::DomLen(_) | EK::DomLo(_) => {
                self.fail("a domain value the link step did not resolve")
            }
            EK::Call(Target::Inst(f), args, site) => {
                // The call's line runs while its arguments are evaluated, as in the reference
                // interpreter, and is the line again after the call.
                let save = self.temp(1);
                self.ops.push(Op::Enter { site: *site, save });
                let c = self.call(*f, args, *site, d, Some(save));
                self.ops.push(Op::Call { c });
            }
            EK::Call(..) => self.fail("an unresolved call reached the engine"),
            EK::CallValue(..) => self.fail("a call through a function value in canonical IR"),
            EK::Intr(op, args, site) => self.intr(*op, args, *site, &e.ty, d),
        }
    }
    fn call(&mut self, f: usize, args: &[Arg<CTy>], site: Site, d: Reg, save: Option<Reg>) -> u32 {
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
            site,
            args: codes,
            d,
            save,
        });
        c
    }
    fn intr(&mut self, op: Intr, args: &[Arg<CTy>], site: Site, ty: &CTy, d: Reg) {
        if op.checked() && site != 0 {
            self.ops.push(Op::At { site });
        }
        let first_ty = match args.first() {
            Some(Arg::Value(v)) => unlent(&v.ty).clone(),
            Some(Arg::View(p) | Arg::Lend(p)) => unlent(&self.place_ty(p)).clone(),
            None => CTy::Unit,
        };
        let elem = |fc: &mut Self, t: &CTy| match unlent(t) {
            CTy::Raw(e) => Some(fc.lay(e)),
            _ => None,
        };
        match op {
            Intr::Add
            | Intr::Sub
            | Intr::Mul
            | Intr::Div
            | Intr::Rem
            | Intr::BitAnd
            | Intr::BitOr
            | Intr::BitXor
            | Intr::Pow
            | Intr::Shl
            | Intr::Shr
            | Intr::Eq
            | Intr::Ne
            | Intr::Lt
            | Intr::Le
            | Intr::Gt
            | Intr::Ge
            | Intr::Below => {
                let Some(&[a, b]) = self.operands(args).as_deref() else {
                    return self.fail("a lent operand");
                };
                let cmp = matches!(
                    op,
                    Intr::Eq | Intr::Ne | Intr::Lt | Intr::Le | Intr::Gt | Intr::Ge | Intr::Below
                );
                let kind = kind(&first_ty);
                self.ops.push(match (cmp, kind) {
                    (true, Kind::Int | Kind::Bool | Kind::U64) => Op::Cmp { op, d, a, b, kind },
                    (true, _) => Op::Fail {
                        rule: "comparison of a value of another kind",
                    },
                    (false, Kind::U64) => Op::Word { op, d, a, b },
                    (false, _) => Op::Int { op, d, a, b },
                });
            }
            Intr::Neg | Intr::Not | Intr::ToU64 | Intr::FromU64 | Intr::CharOf | Intr::CodeOf => {
                let Some(&[a]) = self.operands(args).as_deref() else {
                    return self.fail("a lent operand");
                };
                self.ops.push(match op {
                    Intr::Neg => Op::Neg { d, a },
                    Intr::Not => Op::Not { d, a },
                    Intr::ToU64 => Op::ToU64 { d, a },
                    Intr::FromU64 => Op::FromU64 { d, a },
                    Intr::CharOf => Op::CharOf { d, a },
                    _ => Op::Move { d, s: a, n: 1 },
                });
            }
            Intr::CheckRange | Intr::CheckKey => {
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
            Intr::InputLeft => self.ops.push(Op::InputLeft { d }),
            Intr::ReadInt => self.ops.push(Op::ReadInt { d }),
            Intr::Unset => {}
        }
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
                let t = unlent(&t).clone();
                let lay = self.lay(&t);
                let at = self.c.lays.part(lay, *i).map_or(0, |(at, _)| at);
                off(&mut root, &mut steps, at);
                let part = match (&t, p) {
                    (CTy::Clo(k), _) => self.p.captures(k).get(*i).cloned(),
                    (CTy::Set(ms), Place::Variant(..)) => {
                        ms.get(*i).map(|k| CTy::Clo(Box::new(k.clone())))
                    }
                    (_, Place::Variant(..)) => {
                        self.p.sums.get(&t).and_then(|ps| ps.get(*i).cloned())
                    }
                    _ => self.p.parts(&t).get(*i).cloned(),
                };
                (root, steps, part.unwrap_or(CTy::Error))
            }
            Place::Deref(b) => {
                let (root, mut steps, t) = self.walk(b);
                steps.push(Step::Deref);
                let t = match unlent(&t) {
                    CTy::Ref(t) => (**t).clone(),
                    _ => CTy::Error,
                };
                (root, steps, t)
            }
            Place::Through(b) => {
                let (root, steps, t) = self.walk(b);
                (root, steps, unlent(&t).clone())
            }
            Place::Slot(b, l) => {
                let (root, mut steps, t) = self.walk(b);
                let t = match unlent(&t) {
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
