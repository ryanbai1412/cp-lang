//! Contract 7: the reference interpreter. Written for clarity, not speed. It
//! runs raw instance IR (calls through function values are dynamic) and
//! canonical IR (they are branches), so the two can be compared.
use crate::inst::{CTy, Program};
use crate::ir::*;
use crate::ops::{self, Fault};
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
    Agg(Vec<V>),
    /// Storage is shared until written, so a copy costs nothing until then.
    Raw(Rc<Store>),
    Clo(usize, Vec<V>),
    /// A `var` in a frame still on the stack (`None`: a global).
    Ref(Option<usize>, usize),
    Union(usize, Box<V>),
}

pub struct Trap {
    pub fault: Fault,
    /// The line of the user's file that was running (`ir::Site`).
    pub line: u32,
    pub msg: String,
}
impl Trap {
    /// The trap's code in the diagnostic registry, or `ops::INTERNAL`.
    pub fn code(&self) -> &'static str {
        self.fault.code()
    }
}
pub struct Outcome {
    pub events: Vec<String>,
    pub output: String,
    pub trap: Option<Trap>,
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
    region: Option<Region<'a>>,
    /// A lender whose region left early finishes its own code first, then leaves this way.
    pending: Option<Flow>,
}

struct Machine<'a> {
    p: &'a Program,
    funcs: &'a [Func<CTy>],
    globals: Vec<V>,
    frames: Vec<Frame<'a>>,
    out: String,
    pub events: Vec<String>,
    input: std::vec::IntoIter<String>,
    /// The innermost line of the user's file that is running (`ir::Site`).
    at: Site,
}
type R<T> = Result<T, Trap>;

pub fn run(p: &Program, funcs: &[Func<CTy>], input: &str) -> Outcome {
    let mut m = Machine {
        p,
        funcs,
        globals: vec![V::Unit; p.globals.len()],
        frames: Vec::new(),
        out: String::new(),
        events: Vec::new(),
        input: input
            .split_whitespace()
            .map(String::from)
            .collect::<Vec<_>>()
            .into_iter(),
        at: 0,
    };
    let trap = m
        .call(p.main, Vec::new(), None)
        .err()
        .map(|t| Trap { line: m.at, ..t });
    Outcome {
        events: m.events,
        output: m.out,
        trap,
    }
}

/// The line is filled in where the run ends: it is `Machine::at` then.
fn trap<T>(fault: Fault) -> R<T> {
    Err(fault.into())
}
impl From<Fault> for Trap {
    fn from(fault: Fault) -> Trap {
        Trap {
            fault,
            line: 0,
            msg: fault.message().into(),
        }
    }
}

/// The IR broke a rule the layers before this one keep. Never the program's fault.
fn broken<T>(rule: &'static str) -> R<T> {
    trap(Fault::Internal(rule))
}

impl<'a> Machine<'a> {
    /// Runs a fn. Returns its result, what its lent parameters hold now, and how a region body left, if it did.
    fn call(
        &mut self,
        f: usize,
        args: Vec<V>,
        region: Option<Region<'a>>,
    ) -> R<(V, Vec<V>, Option<Flow>)> {
        let func: &'a Func<CTy> = &self.funcs[f];
        let mut locals = vec![V::Unit; func.locals.len()];
        for ((l, _), a) in func.params.iter().zip(args) {
            locals[*l] = a;
        }
        // Spec 9.6: no depth counting in the language; this stands for the 1 GB stack running out.
        if self.frames.len() >= 4_000_000 {
            return trap(Fault::StackExhausted);
        }
        self.frames.push(Frame {
            locals,
            region,
            pending: None,
        });
        let fp = self.frames.len() - 1;
        let flow = self.block(&func.body, fp)?;
        let Some(mut frame) = self.frames.pop() else {
            return broken("a call returned with no frame");
        };
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
            vals.push(match a {
                Arg::Value(v) => self.expr(v, fp)?,
                // Nothing writes a viewed place during the call, so its value is what the
                // callee would see through the view.
                Arg::View(p) => self.place(fp, p)?.clone(),
                Arg::Lend(_) => V::Unit,
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
                Stmt::Expr(e) => {
                    self.expr(e, fp)?;
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
                Stmt::Loop(body, step) => loop {
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
                            fault: Fault::Check(*kind),
                            line: 0,
                            msg: msg.clone(),
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
                } => {
                    let Target::Inst(f) = target else {
                        return broken("an unresolved region call reached the interpreter");
                    };
                    let vals = self.pass(args, fp)?;
                    let here = self.enter(*site);
                    let (_, back, exit) = self.call(
                        *f,
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
                    let here = self.at;
                    let flow = self.block(r.body, r.fp)?;
                    self.at = here;
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
                    let here = self.at;
                    let flow = self.block(r.body, r.fp)?;
                    self.at = here;
                    let v = std::mem::take(&mut self.frames[r.fp].locals[r.var]);
                    *self.place(fp, p)? = v;
                    // The rest of the lender still runs; the caller leaves after it.
                    if !matches!(flow, Flow::Normal) {
                        self.frames[fp].pending = Some(flow);
                    }
                }
                Stmt::Event(label, vals) => {
                    let mut line = format!("ev {label}");
                    for v in vals {
                        match self.expr(v, fp)? {
                            V::Int(i) => line.push_str(&format!(" {i}")),
                            V::U64(u) => line.push_str(&format!(" {u}")),
                            V::Bool(b) => line.push_str(if b { " true" } else { " false" }),
                            V::Char(c) => {
                                line.push(' ');
                                line.push(char::from(c));
                            }
                            _ => line.push_str(" ?"),
                        }
                    }
                    self.events.push(line);
                }
                Stmt::Print(ps) => {
                    for p in ps {
                        match p {
                            PrintPiece::Text(t) => self.out.push_str(t),
                            PrintPiece::Value(v) => match self.expr(v, fp)? {
                                V::Int(i) => self.out.push_str(&i.to_string()),
                                V::Bool(b) => self.out.push_str(if b { "true" } else { "false" }),
                                V::Char(c) => self.out.push(char::from(c)),
                                V::U64(u) => self.out.push_str(&u.to_string()),
                                _ => return broken("print of a value of another kind"),
                            },
                        }
                    }
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
    fn int(&mut self, a: &'a Arg<CTy>, fp: usize) -> R<i64> {
        let Arg::Value(e) = a else {
            return broken("lent Int");
        };
        match self.expr(e, fp)? {
            V::Int(i) => Ok(i),
            _ => broken("an Int operand of another kind"),
        }
    }
    fn expr(&mut self, e: &'a Expr<CTy>, fp: usize) -> R<V> {
        Ok(match &e.kind {
            EK::Int(i) => V::Int(*i),
            EK::Bool(b) => V::Bool(*b),
            EK::Text(t) => V::Raw(Rc::new(Store::Any(t.bytes().map(V::Char).collect()))),
            EK::Unit => V::Unit,
            EK::DomLen(_) | EK::DomLo(_) => {
                return broken("a domain value the link step did not resolve")
            }
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
                for a in args {
                    vals.push(self.expr(a, fp)?);
                }
                let here = self.enter(*site);
                let v = self.call(f, vals, None)?.0;
                self.at = here;
                v
            }
            EK::Intr(op, args, site) => {
                if op.checked() {
                    self.enter(*site);
                }
                self.intr(*op, args, fp)?
            }
        })
    }
    /// A line of the user's file starts running; inside std (`site` 0) the line stays the caller's.
    fn enter(&mut self, site: Site) -> Site {
        if site != 0 {
            self.at = site;
        }
        self.at
    }
    fn intr(&mut self, op: Intr, args: &'a [Arg<CTy>], fp: usize) -> R<V> {
        Ok(match op {
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
            | Intr::Shr => {
                if let (Arg::Value(x), Arg::Value(y)) = (&args[0], &args[1]) {
                    if x.ty == CTy::U64 {
                        let (a, b) = (self.expr(x, fp)?, self.expr(y, fp)?);
                        return u64_op(op, a, b);
                    }
                }
                let (a, b) = (self.int(&args[0], fp)?, self.int(&args[1], fp)?);
                V::Int(ops::int(op, a, b)?)
            }
            Intr::Neg => V::Int(ops::neg(self.int(&args[0], fp)?)?),
            Intr::Eq | Intr::Ne | Intr::Lt | Intr::Le | Intr::Gt | Intr::Ge | Intr::Below => {
                let (Arg::Value(a), Arg::Value(b)) = (&args[0], &args[1]) else {
                    return broken("lent operand");
                };
                let (a, b) = (self.expr(a, fp)?, self.expr(b, fp)?);
                let (a, b) = match (a, b) {
                    (V::Int(a), V::Int(b)) => (i128::from(a), i128::from(b)),
                    (V::Bool(a), V::Bool(b)) => (i128::from(a), i128::from(b)),
                    (V::U64(a), V::U64(b)) => (i128::from(a), i128::from(b)),
                    _ => return broken("comparison of a value of another kind"),
                };
                V::Bool(ops::compare(op, a, b)?)
            }
            Intr::Not => {
                let Arg::Value(a) = &args[0] else {
                    return broken("lent operand");
                };
                let V::Bool(b) = self.expr(a, fp)? else {
                    return broken("not of a non-Bool");
                };
                V::Bool(!b)
            }
            Intr::RawEmpty => V::Raw(Rc::new(Store::Any(Vec::new()))),
            Intr::RawNew => {
                let n = self.int(&args[0], fp)?;
                let Arg::Value(fill) = &args[1] else {
                    return broken("lent fill");
                };
                let fill = self.expr(fill, fp)?;
                let n = ops::storage_size(n)?;
                V::Raw(Rc::new(match fill {
                    V::Int(x) => Store::Ints(vec![x; n]),
                    fill => Store::Any(vec![fill; n]),
                }))
            }
            Intr::RawGet => {
                let r = match &args[0] {
                    Arg::Value(r) => self.expr(r, fp)?,
                    Arg::View(p) => self.place(fp, p)?.clone(),
                    Arg::Lend(_) => return broken("lent storage in a read"),
                };
                let V::Raw(r) = r else {
                    return broken("read of non-storage");
                };
                let i = self.int(&args[1], fp)?;
                match &*r {
                    Store::Ints(xs) => V::Int(xs[ops::cell(i, xs.len(), false)?]),
                    Store::Any(xs) => xs[ops::cell(i, xs.len(), false)?].clone(),
                }
            }
            Intr::RawSet => {
                let i = self.int(&args[1], fp)?;
                let Arg::Value(x) = &args[2] else {
                    return broken("lent element");
                };
                let x = self.expr(x, fp)?;
                let Arg::Lend(p) = &args[0] else {
                    return broken("storage written without a lend");
                };
                let V::Raw(r) = self.place(fp, p)? else {
                    return broken("write to non-storage");
                };
                let len = match &**r {
                    Store::Ints(xs) => xs.len(),
                    Store::Any(xs) => xs.len(),
                };
                let i = ops::cell(i, len, true)?;
                match (Rc::make_mut(r), x) {
                    (Store::Ints(xs), V::Int(x)) => xs[i] = x,
                    (Store::Any(xs), x) => xs[i] = x,
                    (Store::Ints(_), _) => return broken("a non-integer in integer storage"),
                }
                V::Unit
            }
            Intr::CheckRange | Intr::CheckKey => {
                let Arg::Value(a) = &args[0] else {
                    return broken("lent operand");
                };
                let V::Bool(ok) = self.expr(a, fp)? else {
                    return broken("a check of a non-Bool");
                };
                ops::check(op, ok)?;
                V::Unit
            }
            Intr::InputLeft => V::Bool(!self.input.as_slice().is_empty()),
            Intr::Unset => V::Unit,
            Intr::ToU64 => V::U64(self.int(&args[0], fp)? as u64),
            Intr::FromU64 => {
                let Arg::Value(a) = &args[0] else {
                    return broken("lent operand");
                };
                let V::U64(u) = self.expr(a, fp)? else {
                    return broken("a U64 operand of another kind");
                };
                V::Int(ops::int_of_u64(u)?)
            }
            Intr::CharOf => V::Char(self.int(&args[0], fp)? as u8),
            Intr::CodeOf => {
                let Arg::Value(c) = &args[0] else {
                    return broken("lent Char");
                };
                match self.expr(c, fp)? {
                    V::Char(c) => V::Int(i64::from(c)),
                    _ => return broken("the code of a non-Char"),
                }
            }
            Intr::ReadInt => match self
                .input
                .next()
                .and_then(|t| t.parse::<i64>().ok())
                .filter(|v| *v != i64::MIN)
            {
                Some(i) => V::Int(i),
                None => return trap(Fault::BadIntToken),
            },
        })
    }
}

fn u64_op(op: Intr, a: V, b: V) -> R<V> {
    let b = match (op, b) {
        (Intr::Shl | Intr::Shr, V::Int(b)) => b as u64,
        (Intr::Shl | Intr::Shr, _) => return broken("a shift amount of another kind"),
        (_, V::U64(b)) => b,
        _ => return broken("a U64 operand of another kind"),
    };
    let V::U64(a) = a else {
        return broken("a U64 operand of another kind");
    };
    Ok(V::U64(ops::word(op, a, b)?))
}
