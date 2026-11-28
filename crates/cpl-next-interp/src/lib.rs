//! Contract 7: the reference interpreter. Written for clarity, not speed. It
//! runs raw instance IR (calls through function values are dynamic) and
//! canonical IR (they are branches), so the two can be compared.
use cpl_next_ir::ir::*;
use cpl_next_ir::ops::{self, Fault};
use cpl_next_ir::prog::{CTy, Program};
pub use cpl_next_ir::run::{Outcome, Trap};
use cpl_next_ir::run::{Room, RunInput, Written};
use std::rc::Rc;

/// What a `Raw<T>` holds. Integers are stored flat, so storage of a hundred million of them
/// costs what it costs in the C++.
#[derive(Clone, Debug)]
pub enum Store {
    Ints(Vec<i64>),
    Any(Vec<V>),
}

#[derive(Clone, Debug, Default)]
pub enum V {
    #[default]
    Unit,
    Int(i64),
    Bool(bool),
    Char(u8),
    U64(u64),
    I128(i128),
    F64(f64),
    Agg(Vec<V>),
    /// Storage is shared until written, so a copy costs nothing until then.
    Raw(Rc<Store>),
    Clo(usize, Vec<V>),
    /// A `var` in a frame still on the stack (`None`: a global).
    Ref(Option<usize>, usize),
    Union(usize, Box<V>),
}

enum Flow {
    Normal,
    Break,
    Continue,
    Return(V),
    /// The caller's region body left early, so this generator stops here.
    Exit(Box<Flow>),
}

/// The caller's code a generator or lender runs: where it is, whose frame it runs in, what it binds.
#[derive(Clone, Copy)]
struct Region<'a> {
    fp: usize,
    body: &'a [Stmt<CTy>],
    var: usize,
}
struct Frame<'a> {
    locals: Vec<V>,
    /// Its fn, the frame that called it and the origin of the call: a trap's call stack.
    f: usize,
    parent: Option<usize>,
    origin: u32,
    region: Option<Region<'a>>,
    /// A lender whose region left early finishes its own code first, then leaves this way.
    pending: Option<Flow>,
}

struct Machine<'a> {
    p: &'a Program,
    funcs: &'a [Func<CTy>],
    globals: Vec<V>,
    frames: Vec<Frame<'a>>,
    out: Written,
    input: ops::Input,
    rand: u64,
    /// The origin of the innermost site of the user's file that is running.
    at: u32,
    /// The frame whose code is running: a region body runs in its caller's.
    cur: usize,
    room: Room,
}
type R<T> = Result<T, Trap>;

pub fn run(p: &Program, funcs: &[Func<CTy>], input: RunInput) -> Outcome {
    let mut m = Machine {
        p,
        funcs,
        globals: vec![V::Unit; p.globals.len()],
        frames: Vec::new(),
        out: Written::to(input.out),
        input: input.io.input(),
        rand: ops::RAND_SEED,
        at: 0,
        cur: 0,
        room: Room::new(p, input.limits),
    };
    let trap = m.call(p.main, Vec::new(), None).err().map(|t| Trap {
        origin: m.at,
        calls: m.calls(),
        ..t
    });
    m.out.end(trap)
}

/// The origin is filled in where the run ends: it is `Machine::at` then.
fn trap<T>(fault: Fault) -> R<T> {
    Err(fault.into())
}

/// The IR broke a rule the layers before this one keep. Never the program's fault.
fn broken<T>(rule: &'static str) -> R<T> {
    trap(Fault::Internal(rule))
}

impl<'a> Machine<'a> {
    /// The calls a trap left, innermost first: each frame's fn and the origin running in it.
    fn calls(&self) -> Vec<(usize, u32)> {
        let mut calls = Vec::new();
        let (mut k, mut o) = (Some(self.cur), self.at);
        while let Some(fr) = k.and_then(|i| self.frames.get(i)) {
            calls.push((fr.f, o));
            (k, o) = (fr.parent, fr.origin);
        }
        calls
    }
    /// Runs a fn. Returns its result, what its lent parameters hold now, and how a region body left, if it did.
    /// A call of `f`. It recurses natively, its host frames larger than the run's counted words,
    /// so the host stack grows in segments (`stacker`) and R0601 (`Stack::enter`) runs out first.
    fn call(
        &mut self,
        f: usize,
        args: Vec<V>,
        region: Option<Region<'a>>,
    ) -> R<(V, Vec<V>, Option<Flow>)> {
        stacker::maybe_grow(1 << 20, 64 << 20, || {
            let func: &'a Func<CTy> = &self.funcs[f];
            let mut locals = vec![V::Unit; func.locals.len()];
            for ((l, _), a) in func.params.iter().zip(args) {
                locals[*l] = a;
            }
            // Spec 9.6: no depth counting in the language; the stack runs out at the run's limit.
            self.room.stack.enter(f, self.frames.len())?;
            let parent = (!self.frames.is_empty()).then_some(self.cur);
            self.frames.push(Frame {
                locals,
                f,
                parent,
                origin: self.at,
                region,
                pending: None,
            });
            let fp = self.frames.len() - 1;
            self.cur = fp;
            let flow = self.block(&func.body, fp)?;
            let Some(mut frame) = self.frames.pop() else {
                return broken("a call returned with no frame");
            };
            self.cur = frame.parent.unwrap_or(0);
            let (ret, exit) = match flow {
                Flow::Return(v) => (v, frame.pending.take()),
                Flow::Exit(f) => (V::Unit, Some(*f)),
                _ => (V::Unit, frame.pending.take()),
            };
            // What the lent parameters hold now goes back to the caller's places.
            let back = func
                .params
                .iter()
                .filter(|(_, lent)| *lent)
                .map(|(l, _)| std::mem::take(&mut frame.locals[*l]))
                .collect();
            Ok((ret, back, exit))
        })
    }
    fn place(&mut self, fp: usize, p: &Place) -> R<&mut V> {
        Ok(match p {
            Place::Local(i) => &mut self.frames[fp].locals[*i],
            Place::Global(i) => &mut self.globals[*i],
            Place::Field(b, i) => match self.place(fp, b)? {
                V::Agg(fs) | V::Clo(_, fs) => &mut fs[*i],
                _ => return broken("field of a value of another kind"),
            },
            // A lent slot holds its source's shared storage, so reading through it is the slot.
            Place::Through(b) => self.place(fp, b)?,
            Place::Deref(b) => match self.place(fp, b)?.clone() {
                V::Ref(Some(f), l) => &mut self.frames[f].locals[l],
                V::Ref(None, g) => &mut self.globals[g],
                _ => return broken("deref of a value of another kind"),
            },
            Place::Variant(b, _) => match self.place(fp, b)? {
                V::Union(_, v) => v,
                _ => return broken("variant of a value of another kind"),
            },
            Place::Slot(b, l) => {
                let V::Int(i) = self.frames[fp].locals[*l] else {
                    return broken("a cell index that is not an Int");
                };
                let V::Raw(r) = self.place(fp, b)? else {
                    return broken("a cell of a value that is not storage");
                };
                // A cell handed out as a place holds a value, so integer storage widens to values.
                let s = Rc::make_mut(r);
                if let Store::Ints(xs) = s {
                    *s = Store::Any(std::mem::take(xs).into_iter().map(V::Int).collect());
                }
                let Store::Any(xs) = s else {
                    return broken("storage that did not widen");
                };
                match usize::try_from(i).ok().and_then(|i| xs.get_mut(i)) {
                    Some(v) => v,
                    None => return broken("a cell outside its storage"),
                }
            }
        })
    }
    /// Evaluates arguments: values first, then each lent place is handed over.
    fn pass(&mut self, args: &'a [Arg<CTy>], fp: usize) -> R<Vec<V>> {
        let mut vals = Vec::with_capacity(args.len());
        for a in args {
            // Nothing writes a viewed place during the call, so its value is what the callee
            // would see through the view.
            vals.push(match a {
                Arg::Lend(_) => V::Unit,
                a => self.arg(a, fp)?,
            });
        }
        for (i, a) in args.iter().enumerate() {
            if let Arg::Lend(p) = a {
                vals[i] = std::mem::take(self.place(fp, p)?);
            }
        }
        Ok(vals)
    }
    fn hand_back(&mut self, args: &'a [Arg<CTy>], fp: usize, back: Vec<V>) -> R<()> {
        let lent = args.iter().filter_map(|a| match a {
            Arg::Lend(p) => Some(p),
            Arg::Value(_) | Arg::View(_) => None,
        });
        for (p, v) in lent.zip(back) {
            *self.place(fp, p)? = v;
        }
        Ok(())
    }
    fn block(&mut self, b: &'a [Stmt<CTy>], fp: usize) -> R<Flow> {
        for s in b {
            match s {
                Stmt::Let(l, e) => self.frames[fp].locals[*l] = self.expr(e, fp)?,
                Stmt::Assign(p, e) => {
                    let v = self.expr(e, fp)?;
                    *self.place(fp, p)? = v;
                }
                Stmt::Expr(e) => _ = self.expr(e, fp)?,
                Stmt::Drop(p) => *self.place(fp, p)? = V::Unit,
                Stmt::Fact(c) => {
                    if !matches!(self.expr(c, fp)?, V::Bool(true)) {
                        return broken("a fact lowering states is false");
                    }
                }
                Stmt::If(c, a, b2) => {
                    let V::Bool(c) = self.expr(c, fp)? else {
                        return broken("if on a non-Bool");
                    };
                    match self.block(if c { a } else { b2 }, fp)? {
                        Flow::Normal => {}
                        f => return Ok(f),
                    }
                }
                Stmt::Loop(body, step, _) => loop {
                    match self.block(body, fp)? {
                        Flow::Break => break,
                        f @ (Flow::Return(_) | Flow::Exit(_)) => return Ok(f),
                        Flow::Normal | Flow::Continue => {}
                    }
                    self.block(step, fp)?;
                },
                Stmt::Switch(c, arms) => {
                    let V::Int(i) = self.expr(c, fp)? else {
                        return broken("switch on a non-Int");
                    };
                    match self.block(&arms[i as usize], fp)? {
                        Flow::Normal => {}
                        f => return Ok(f),
                    }
                }
                Stmt::Break => return Ok(Flow::Break),
                Stmt::Continue => return Ok(Flow::Continue),
                Stmt::Return(e) => {
                    let v = match e {
                        Some(e) => self.expr(e, fp)?,
                        None => V::Unit,
                    };
                    return Ok(Flow::Return(v));
                }
                Stmt::Check {
                    kind,
                    cond,
                    msg,
                    site,
                } => {
                    let V::Bool(ok) = self.expr(cond, fp)? else {
                        return broken("check on a non-Bool");
                    };
                    if !ok {
                        self.enter(*site);
                        return Err(Trap {
                            msg: msg.clone(),
                            ..Fault::Check(*kind).into()
                        });
                    }
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
                    // A generator value's walk calls the closure it holds, captures first.
                    let (f, mut vals) = match target {
                        Target::Inst(f) => (*f, Vec::new()),
                        Target::Value(c) => match self.expr(c, fp)? {
                            V::Clo(f, caps) => (f, caps),
                            _ => return broken("a walk of a non-function"),
                        },
                        _ => return broken("an unresolved region call reached the interpreter"),
                    };
                    vals.extend(self.pass(args, fp)?);
                    let here = self.enter(*site);
                    let (_, back, exit) = self.call(
                        f,
                        vals,
                        Some(Region {
                            fp,
                            body,
                            var: *var,
                        }),
                    )?;
                    self.at = here;
                    self.hand_back(args, fp, back)?;
                    match exit {
                        None => {}
                        // A generator's region is a loop body: `break` ends the loop.
                        Some(Flow::Break) if !*lend => {}
                        Some(f) => return Ok(f),
                    }
                }
                Stmt::Yield(e) => {
                    let v = self.expr(e, fp)?;
                    let Some(r) = self.frames[fp].region else {
                        return broken("yield with no caller code");
                    };
                    self.frames[r.fp].locals[r.var] = v;
                    let (here, was) = (self.at, self.cur);
                    self.cur = r.fp;
                    let flow = self.block(r.body, r.fp)?;
                    (self.at, self.cur) = (here, was);
                    match flow {
                        Flow::Normal | Flow::Continue => {}
                        other => return Ok(Flow::Exit(Box::new(other))),
                    }
                }
                Stmt::LendOut(p) => {
                    let Some(r) = self.frames[fp].region else {
                        return broken("lend with no caller code");
                    };
                    let v = std::mem::take(self.place(fp, p)?);
                    self.frames[r.fp].locals[r.var] = v;
                    let (here, was) = (self.at, self.cur);
                    self.cur = r.fp;
                    let flow = self.block(r.body, r.fp)?;
                    (self.at, self.cur) = (here, was);
                    let v = std::mem::take(&mut self.frames[r.fp].locals[r.var]);
                    *self.place(fp, p)? = v;
                    // The rest of the lender still runs; the caller leaves after it.
                    if !matches!(flow, Flow::Normal) {
                        self.frames[fp].pending = Some(flow);
                    }
                }
                Stmt::Event(_) if self.out.showing() => {}
                Stmt::Event(ev) => {
                    for v in &ev.vals {
                        self.out.mark();
                        self.expr(v, fp)?;
                    }
                    let n = ev.vals.len();
                    self.out.fire(ev.kind.clone(), ev.func, ev.site, n);
                }
            }
        }
        Ok(Flow::Normal)
    }
    fn do_call(&mut self, f: usize, args: &'a [Arg<CTy>], fp: usize) -> R<V> {
        let vals = self.pass(args, fp)?;
        let (ret, back, _) = self.call(f, vals, None)?;
        self.hand_back(args, fp, back)?;
        Ok(ret)
    }
    /// An operand's value: a value, or a viewed place's.
    fn arg(&mut self, a: &'a Arg<CTy>, fp: usize) -> R<V> {
        match a {
            Arg::Value(e) => self.expr(e, fp),
            Arg::View(p) => Ok(self.place(fp, p)?.clone()),
            Arg::Lend(_) => broken("lent operand"),
        }
    }
    /// The storage a lent operand names, to take from or write.
    fn lent_raw(&mut self, a: &'a Arg<CTy>, fp: usize) -> R<&mut Rc<Store>> {
        match a {
            Arg::Lend(p) => match self.place(fp, p)? {
                V::Raw(r) => Ok(r),
                _ => broken("a lent operand that is not storage"),
            },
            _ => broken("storage written without a lend"),
        }
    }
    fn int(&mut self, a: &'a Arg<CTy>, fp: usize) -> R<i64> {
        match self.arg(a, fp)? {
            V::Int(i) => Ok(i),
            _ => broken("an Int operand of another kind"),
        }
    }
    fn bool(&mut self, a: &'a Arg<CTy>, fp: usize) -> R<bool> {
        match self.arg(a, fp)? {
            V::Bool(b) => Ok(b),
            _ => broken("a Bool operand of another kind"),
        }
    }
    fn expr(&mut self, e: &'a Expr<CTy>, fp: usize) -> R<V> {
        Ok(match &e.kind {
            EK::Int(i) => V::Int(*i),
            EK::F64(b) => V::F64(f64::from_bits(*b)),
            EK::Bool(b) => V::Bool(*b),
            EK::Text(t) => V::Raw(Rc::new(Store::Any(t.bytes().map(V::Char).collect()))),
            EK::Unit => V::Unit,
            EK::DomLen(_) | EK::DomLo(_) | EK::DomHeld(_) | EK::DomCounted(_) => {
                return broken("a domain value the link step did not resolve")
            }
            EK::HasBound(..) => return broken("a bound test the link step did not decide"),
            EK::Pass(..) => return broken("a place passed before ownership decided how"),
            EK::Copy(p) | EK::Lent(p) => self.place(fp, p)?.clone(),
            EK::AddrOf(p) => match p {
                Place::Local(l) => V::Ref(Some(fp), *l),
                Place::Global(g) => V::Ref(None, *g),
                _ => return broken("a reference to part of a variable"),
            },
            EK::Tag(p) => match self.place(fp, p)? {
                V::Union(i, _) => V::Int(*i as i64),
                _ => return broken("tag of a value of another kind"),
            },
            EK::MakeStruct(xs) => {
                let mut fs = Vec::with_capacity(xs.len());
                for x in xs {
                    fs.push(self.expr(x, fp)?);
                }
                V::Agg(fs)
            }
            EK::MakeClosure(xs) => {
                let mut fs = Vec::with_capacity(xs.len());
                for x in xs {
                    fs.push(self.expr(x, fp)?);
                }
                let CTy::Clo(k) = &e.ty else {
                    return broken("closure of a non-closure type");
                };
                V::Clo(self.p.index[&**k], fs)
            }
            // In raw IR a function value is the closure itself: which set it sits in changes nothing.
            EK::Inject(x) | EK::Widen(x) => self.expr(x, fp)?,
            EK::MakeUnion(i, x) => V::Union(*i, Box::new(self.expr(x, fp)?)),
            EK::Call(Target::Inst(f), args, site) => {
                let here = self.enter(*site);
                let v = self.do_call(*f, args, fp)?;
                self.at = here;
                v
            }
            EK::Call(..) => return broken("an unresolved call reached the interpreter"),
            EK::CallValue(c, args, site) => {
                let V::Clo(f, caps) = self.expr(c, fp)? else {
                    return broken("call of a non-function");
                };
                let mut vals = caps;
                vals.extend(self.pass(args, fp)?);
                let here = self.enter(*site);
                let (v, back, _) = self.call(f, vals, None)?;
                self.at = here;
                self.hand_back(args, fp, back)?;
                v
            }
            EK::Intr(op, args, site) => {
                if op.checked() {
                    self.enter(*site);
                }
                self.intr(*op, args, fp, &e.ty)?
            }
        })
    }
    /// A site of the user's file starts running; inside std (origin 0) the origin stays the
    /// caller's.
    fn enter(&mut self, site: Site) -> u32 {
        let o = self.funcs[self.frames[self.cur].f].origins[site as usize];
        if o != 0 {
            self.at = o;
        }
        self.at
    }
    fn intr(&mut self, op: Intr, args: &'a [Arg<CTy>], fp: usize, ty: &CTy) -> R<V> {
        if ops::on_scalars(op) {
            let mut xs = [ops::Scalar::Bool(false); 2];
            for (x, a) in xs.iter_mut().zip(args) {
                *x =
                    scalar(&self.arg(a, fp)?).map_or_else(|| broken("a non-scalar operand"), Ok)?;
            }
            let Some(v) = ops::scalar(op, &xs[..args.len().min(2)]) else {
                return broken("operands of kinds the operation does not take");
            };
            return Ok(value(v?));
        }
        Ok(match op {
            Intr::RawEmpty => V::Raw(Rc::new(Store::Any(Vec::new()))),
            Intr::RawNew | Intr::RawCap => {
                let n = self.int(&args[0], fp)?;
                let fill = match op {
                    Intr::RawNew => self.arg(&args[1], fp)?,
                    _ => V::Unit,
                };
                let n = self.room.cells(self.p, ty, n)?;
                V::Raw(Rc::new(match fill {
                    V::Int(x) => Store::Ints(vec![x; n]),
                    fill => Store::Any(vec![fill; n]),
                }))
            }
            Intr::RawTake => {
                let i = self.int(&args[1], fp)?;
                match Rc::make_mut(self.lent_raw(&args[0], fp)?) {
                    Store::Ints(xs) => V::Int(xs[ops::cell(i, xs.len(), false)?]),
                    Store::Any(xs) => {
                        let i = ops::cell(i, xs.len(), false)?;
                        std::mem::replace(&mut xs[i], V::Unit)
                    }
                }
            }
            Intr::RawGet => {
                let V::Raw(r) = self.arg(&args[0], fp)? else {
                    return broken("read of non-storage");
                };
                let i = self.int(&args[1], fp)?;
                match &*r {
                    Store::Ints(xs) => V::Int(xs[ops::cell(i, xs.len(), false)?]),
                    Store::Any(xs) => xs[ops::cell(i, xs.len(), false)?].clone(),
                }
            }
            Intr::Out => {
                let n = self.int(&args[1], fp)?;
                let V::Raw(r) = self.arg(&args[0], fp)? else {
                    return broken("output of non-storage");
                };
                let Store::Any(cs) = &*r else {
                    return broken("output of storage of another kind");
                };
                for c in cs.iter().take(ops::out_len(n, cs.len())) {
                    let V::Char(c) = c else {
                        return broken("output of a value that is not a character");
                    };
                    self.out.text.push(char::from(*c));
                }
                V::Unit
            }
            Intr::RawSet => {
                let i = self.int(&args[1], fp)?;
                let x = self.arg(&args[2], fp)?;
                match (Rc::make_mut(self.lent_raw(&args[0], fp)?), x) {
                    (Store::Ints(xs), V::Int(x)) => {
                        ops::cell(i, xs.len(), true).map(|i| xs[i] = x)?
                    }
                    (Store::Any(xs), x) => ops::cell(i, xs.len(), true).map(|i| xs[i] = x)?,
                    (Store::Ints(_), _) => return broken("a non-integer in integer storage"),
                }
                V::Unit
            }
            Intr::CheckRange
            | Intr::CheckKey
            | Intr::CheckVariant
            | Intr::CheckToken
            | Intr::CheckInput => {
                ops::check(op, self.bool(&args[0], fp)?)?;
                V::Unit
            }
            Intr::InputLeft => V::Bool(self.input.left(&mut self.out)),
            Intr::ReadByte => V::Int(self.input.byte(&mut self.out)),
            Intr::PeekByte => {
                let k = self.int(&args[0], fp)?;
                V::Int(self.input.peek(&mut self.out, k))
            }
            Intr::RandWord => V::U64(ops::rand_word(&mut self.rand)),
            Intr::Flush => {
                self.out.flush();
                V::Unit
            }
            Intr::Unset => V::Unit,
            Intr::ReadInt => {
                let t = self.input.token(&mut self.out);
                V::Int(ops::read_int(t.as_deref()).map_err(|f| Trap::at_token(f, t.as_deref()))?)
            }
            _ => return broken("a scalar operation outside `ops::scalar`"),
        })
    }
}

fn scalar(v: &V) -> Option<ops::Scalar> {
    Some(match *v {
        V::Int(i) => ops::Scalar::Int(i),
        V::U64(u) => ops::Scalar::U64(u),
        V::I128(w) => ops::Scalar::I128(w),
        V::F64(x) => ops::Scalar::F64(x.to_bits()),
        V::Bool(b) => ops::Scalar::Bool(b),
        V::Char(c) => ops::Scalar::Char(c),
        _ => return None,
    })
}

fn value(s: ops::Scalar) -> V {
    match s {
        ops::Scalar::Int(i) => V::Int(i),
        ops::Scalar::U64(u) => V::U64(u),
        ops::Scalar::I128(w) => V::I128(w),
        ops::Scalar::F64(x) => V::F64(f64::from_bits(x)),
        ops::Scalar::Bool(b) => V::Bool(b),
        ops::Scalar::Char(c) => V::Char(c),
    }
}
