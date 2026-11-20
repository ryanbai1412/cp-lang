//! Lowering: from surface terms to polynomial constraints.
//!
//! Most of the prover's strength is here, because a law that becomes a
//! syntactic identity needs no search at all:
//!
//! - integer arithmetic is normalised as polynomials, so `+` and `*` are
//!   associative, commutative and distributive by construction;
//! - `min` and `max` are flattened, sorted and distributed over `+`
//!   (`a + max(b, c)` is `max(a + b, a + c)`), which is the whole algebra
//!   of segment-tree monoids;
//! - `%` is reduced modulo its divisor before it is named, so modular
//!   products and sums associate;
//! - `^`, `&`, `|` and `gcd` are flattened, sorted and folded.
//!
//! Whatever is left (an `if`, a quotient, an array element) becomes a
//! fresh variable with a defining constraint the solver picks up lazily.

use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use crate::core::{fit, Formula, Lazy, Res};
use crate::eval::is_builtin;
use crate::poly::{Mono, Poly, Var};
use crate::term::{BinOp, Error, Sort, Term, UnOp, INT_MAX, INT_MIN};

/// A symbolic value: what a term denotes for every input at once.
#[derive(Clone)]
pub(crate) enum SVal {
    Int(Poly),
    Bool(Formula),
    Tuple(Vec<(Option<String>, SVal)>),
    /// An optional: `value` only means something where `some` holds.
    Opt {
        some: Formula,
        value: Box<SVal>,
    },
    /// `none`, before its type is known.
    NoneLit,
    Lambda(Rc<Closure>),
    /// A builtin or a declared function.
    Named(String),
}

pub(crate) struct Closure {
    params: Vec<String>,
    body: Term,
    scope: Scope,
}

type Scope = Option<Rc<Frame>>;

struct Frame {
    name: String,
    value: SVal,
    parent: Scope,
}

impl SVal {
    /// The integer variables and polynomials the value is made of.
    pub fn leaves(&self, into: &mut Vec<Poly>) {
        match self {
            SVal::Int(p) => into.push(p.clone()),
            SVal::Bool(f) => {
                if let Formula::Ge(p) = f {
                    into.push(p.clone());
                }
            }
            SVal::Tuple(items) => items.iter().for_each(|(_, item)| item.leaves(into)),
            SVal::Opt { some, value } => {
                SVal::Bool(some.clone()).leaves(into);
                value.leaves(into);
            }
            SVal::NoneLit | SVal::Lambda(_) | SVal::Named(_) => {}
        }
    }

    pub fn vars(&self) -> Vec<Var> {
        let mut leaves = Vec::new();
        self.leaves(&mut leaves);
        leaves.iter().flat_map(|p| p.vars()).collect()
    }

    fn kind(&self) -> &'static str {
        match self {
            SVal::Int(_) => "an Int",
            SVal::Bool(_) => "a Bool",
            SVal::Tuple(_) => "a tuple",
            SVal::Opt { .. } | SVal::NoneLit => "an optional",
            SVal::Lambda(_) | SVal::Named(_) => "a function",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Ac {
    Min,
    Max,
    Xor,
    And,
    Or,
    Gcd,
}

/// One use of a declared array or function.
#[derive(Clone)]
pub(crate) struct App {
    pub symbol: String,
    pub args: Vec<SVal>,
    flat: Vec<Poly>,
    pub result: SVal,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Hazard {
    Overflow,
    DivByZero,
    Unwrap,
}

/// Something that must hold for a term to evaluate without a trap.
#[derive(Clone, Debug)]
pub(crate) struct Obligation {
    pub hazard: Hazard,
    /// The condition under which the operation runs.
    pub path: Formula,
    /// What must hold there.
    pub safe: Formula,
    /// The value that must stay an `Int`, for overflow.
    pub value: Option<Poly>,
    pub text: String,
}

/// The variables and definitions of one context; it only ever grows.
#[derive(Clone, Default)]
pub(crate) struct Store {
    /// A display name for every core variable.
    pub names: Vec<String>,
    pub lazies: Vec<Lazy>,
    pub apps: Vec<App>,
    ac: HashMap<(Ac, Vec<Poly>), Var>,
    ac_args: HashMap<Var, (Ac, Vec<Poly>)>,
    divmods: HashMap<(Poly, Poly), (Var, Var)>,
    /// For a remainder variable, the dividend and the divisor it came from.
    remainders: HashMap<Var, (Poly, Poly)>,
    opaque: HashMap<(&'static str, Vec<Poly>), Var>,
    ites: HashMap<(Formula, Vec<Poly>, Vec<Poly>), Vec<Var>>,
}

impl Store {
    pub fn fresh(&mut self, name: impl Into<String>) -> Var {
        self.names.push(name.into());
        (self.names.len() - 1) as Var
    }

    fn define(&mut self, vars: &[Var], formula: Formula) {
        self.lazies.push(Lazy {
            triggers: vec![vars.to_vec()],
            formula,
        });
    }
}

/// The names a term can mention.
#[derive(Clone, Default)]
pub(crate) struct Globals {
    pub vars: HashMap<String, SVal>,
    pub defs: HashMap<String, Term>,
    /// Declared arrays and functions, with the type of their results.
    pub symbols: HashMap<String, Sort>,
}

pub(crate) struct Lowerer<'a> {
    pub store: &'a mut Store,
    pub globals: &'a Globals,
    path: Vec<Formula>,
    depth: usize,
    /// Set to collect what the term needs in order not to trap.
    pub obligations: Option<Vec<Obligation>>,
}

fn int(c: i128) -> Poly {
    Poly::constant(c)
}

fn bool_of(var: Var) -> Formula {
    Formula::ge(Poly::var(var).add_const(-1).unwrap())
}

fn in_int_range(p: &Poly) -> Res<Formula> {
    Ok(Formula::and([
        Formula::le(&int(INT_MIN), p)?,
        Formula::le(p, &int(INT_MAX))?,
    ]))
}

/// A value of the same shape, for the payload of an absent optional.
fn blank(like: &SVal) -> SVal {
    match like {
        SVal::Int(_) => SVal::Int(Poly::zero()),
        SVal::Bool(_) => SVal::Bool(Formula::False),
        SVal::Tuple(items) => SVal::Tuple(
            items
                .iter()
                .map(|(name, item)| (name.clone(), blank(item)))
                .collect(),
        ),
        SVal::Opt { value, .. } => SVal::Opt {
            some: Formula::False,
            value: Box::new(blank(value)),
        },
        other => other.clone(),
    }
}

impl<'a> Lowerer<'a> {
    pub fn new(store: &'a mut Store, globals: &'a Globals) -> Lowerer<'a> {
        Lowerer {
            store,
            globals,
            path: Vec::new(),
            depth: 0,
            obligations: None,
        }
    }

    pub fn term(&mut self, term: &Term) -> Res<SVal> {
        self.lower(term, &None)
    }

    pub fn formula(&mut self, term: &Term) -> Res<Formula> {
        match self.term(term)? {
            SVal::Bool(f) => Ok(f),
            other => Err(Error::new(format!(
                "`{term}` is {}, not a Bool",
                other.kind()
            ))),
        }
    }

    pub fn poly(&mut self, term: &Term) -> Res<Poly> {
        match self.term(term)? {
            SVal::Int(p) => Ok(p),
            other => Err(Error::new(format!(
                "`{term}` is {}, not an Int",
                other.kind()
            ))),
        }
    }

    /// A fresh unknown of type `sort`, with the facts its type gives.
    pub fn fresh(&mut self, name: &str, sort: &Sort) -> Res<(SVal, Formula)> {
        Ok(match sort {
            Sort::Int => {
                let p = Poly::var(self.store.fresh(name));
                let facts = in_int_range(&p)?;
                (SVal::Int(p), facts)
            }
            Sort::Bool => {
                let v = self.store.fresh(name);
                let p = Poly::var(v);
                let facts = Formula::and([Formula::le(&int(0), &p)?, Formula::le(&p, &int(1))?]);
                (SVal::Bool(bool_of(v)), facts)
            }
            Sort::Range { lo, hi, inclusive } => {
                let (lo, hi) = (self.poly(lo)?, self.poly(hi)?);
                let p = Poly::var(self.store.fresh(name));
                let upper = if *inclusive {
                    Formula::le(&p, &hi)?
                } else {
                    Formula::lt(&p, &hi)?
                };
                let facts = Formula::and([Formula::le(&lo, &p)?, upper]);
                (SVal::Int(p), facts)
            }
            Sort::Tuple(sorts) => {
                let (mut items, mut facts) = (Vec::new(), Vec::new());
                for (i, (field, sort)) in sorts.iter().enumerate() {
                    let label = field.clone().unwrap_or_else(|| i.to_string());
                    let (item, fact) = self.fresh(&format!("{name}.{label}"), sort)?;
                    items.push((field.clone(), item));
                    facts.push(fact);
                }
                (SVal::Tuple(items), Formula::and(facts))
            }
            Sort::Opt(inner) => {
                let flag = self.store.fresh(format!("{name}.some"));
                let p = Poly::var(flag);
                let (value, fact) = self.fresh(name, inner)?;
                let facts =
                    Formula::and([Formula::le(&int(0), &p)?, Formula::le(&p, &int(1))?, fact]);
                (
                    SVal::Opt {
                        some: bool_of(flag),
                        value: Box::new(value),
                    },
                    facts,
                )
            }
            Sort::Named(ty) => return Err(Error::new(format!("the type `{ty}` is not bound"))),
        })
    }

    fn here(&self) -> Formula {
        Formula::and(self.path.iter().cloned())
    }

    fn oblige(&mut self, hazard: Hazard, safe: Formula, value: Option<Poly>, term: &Term) {
        if safe == Formula::True {
            return;
        }
        let path = self.here();
        if let Some(list) = self.obligations.as_mut() {
            list.push(Obligation {
                hazard,
                path,
                safe,
                value,
                text: term.to_string(),
            });
        }
    }

    fn under<T>(&mut self, cond: Formula, run: impl FnOnce(&mut Self) -> Res<T>) -> Res<T> {
        self.path.push(cond);
        let out = run(self);
        self.path.pop();
        out
    }

    fn lower_bool(&mut self, term: &Term, scope: &Scope) -> Res<Formula> {
        match self.lower(term, scope)? {
            SVal::Bool(f) => Ok(f),
            other => Err(Error::new(format!(
                "`{term}` is {}, not a Bool",
                other.kind()
            ))),
        }
    }

    fn lower_int(&mut self, term: &Term, scope: &Scope) -> Res<Poly> {
        match self.lower(term, scope)? {
            SVal::Int(p) => Ok(p),
            other => Err(Error::new(format!(
                "`{term}` is {}, not an Int",
                other.kind()
            ))),
        }
    }

    fn lower(&mut self, term: &Term, scope: &Scope) -> Res<SVal> {
        self.depth += 1;
        if self.depth > 256 {
            return Err(Error::new(
                "the term nests too deeply (a definition may refer to itself)",
            ));
        }
        let out = self.lower_inner(term, scope);
        self.depth -= 1;
        out
    }

    fn lower_inner(&mut self, term: &Term, scope: &Scope) -> Res<SVal> {
        Ok(match term {
            Term::Int(v) => SVal::Int(int(*v)),
            Term::Bool(b) => SVal::Bool(if *b { Formula::True } else { Formula::False }),
            Term::None => SVal::NoneLit,
            Term::Var(name) => return self.name(name, scope),
            Term::Unary(UnOp::Neg, x) => SVal::Int(fit(self.lower_int(x, scope)?.neg())?),
            Term::Unary(UnOp::Not, x) => SVal::Bool(self.lower_bool(x, scope)?.negate()?),
            Term::Unary(UnOp::Unwrap, x) => match self.lower(x, scope)? {
                SVal::Opt { some, value } => {
                    self.oblige(Hazard::Unwrap, some, None, term);
                    *value
                }
                SVal::NoneLit => return Err(Error::new(format!("`{term}` always traps"))),
                present => present,
            },
            Term::Binary(op, lhs, rhs) => return self.binary(*op, lhs, rhs, scope, term),
            Term::If(cond, then, other) => {
                let cond = self.lower_bool(cond, scope)?;
                match cond {
                    Formula::True => return self.lower(then, scope),
                    Formula::False => return self.lower(other, scope),
                    _ => {}
                }
                let then = self.under(cond.clone(), |l| l.lower(then, scope))?;
                let other = self.under(cond.negate()?, |l| l.lower(other, scope))?;
                self.ite(&cond, then, other)?
            }
            Term::Call(callee, args) => {
                let callee = self.lower(callee, scope)?;
                let mut values = Vec::with_capacity(args.len());
                for arg in args {
                    values.push(self.lower(arg, scope)?);
                }
                return self.call(callee, values, term);
            }
            Term::Index(base, args) => {
                let symbol = match &**base {
                    Term::Var(name) if self.globals.symbols.contains_key(name) => name.clone(),
                    _ => return Err(Error::new(format!("`{base}` is not a declared array"))),
                };
                let mut values = Vec::with_capacity(args.len());
                for arg in args {
                    values.push(self.lower(arg, scope)?);
                }
                self.apply(&symbol, values, term)?
            }
            Term::Tuple(items) => {
                let mut values = Vec::with_capacity(items.len());
                for (name, item) in items {
                    values.push((name.clone(), self.lower(item, scope)?));
                }
                SVal::Tuple(values)
            }
            Term::Field(base, field) => match self.lower(base, scope)? {
                SVal::Tuple(mut items) => {
                    let by_name = items
                        .iter()
                        .position(|(name, _)| name.as_deref() == Some(field));
                    let index = by_name.or_else(|| field.parse().ok().filter(|i| *i < items.len()));
                    match index {
                        Some(i) => items.swap_remove(i).1,
                        None => return Err(Error::new(format!("`{base}` has no field `{field}`"))),
                    }
                }
                _ => return Err(Error::new(format!("`{base}` has no field `{field}`"))),
            },
            Term::Lambda(params, body) => SVal::Lambda(Rc::new(Closure {
                params: params.clone(),
                body: (**body).clone(),
                scope: scope.clone(),
            })),
            Term::InRange(x, lo, hi, inclusive) => {
                let x = self.lower_int(x, scope)?;
                let (lo, hi) = (self.lower_int(lo, scope)?, self.lower_int(hi, scope)?);
                let upper = if *inclusive {
                    Formula::le(&x, &hi)?
                } else {
                    Formula::lt(&x, &hi)?
                };
                SVal::Bool(Formula::and([Formula::le(&lo, &x)?, upper]))
            }
        })
    }

    fn name(&mut self, name: &str, scope: &Scope) -> Res<SVal> {
        let mut frame = scope.as_ref();
        while let Some(f) = frame {
            if f.name == name {
                return Ok(f.value.clone());
            }
            frame = f.parent.as_ref();
        }
        if let Some(value) = self.globals.vars.get(name) {
            return Ok(value.clone());
        }
        if let Some(def) = self.globals.defs.get(name) {
            return self.lower(def, &None);
        }
        if self.globals.symbols.contains_key(name) || is_builtin(name) {
            return Ok(SVal::Named(name.to_string()));
        }
        Err(Error::new(format!("`{name}` is not declared")))
    }

    fn binary(
        &mut self,
        op: BinOp,
        lhs: &Term,
        rhs: &Term,
        scope: &Scope,
        term: &Term,
    ) -> Res<SVal> {
        match op {
            BinOp::And => {
                let l = self.lower_bool(lhs, scope)?;
                if l == Formula::False {
                    return Ok(SVal::Bool(l));
                }
                let r = self.under(l.clone(), |s| s.lower_bool(rhs, scope))?;
                return Ok(SVal::Bool(Formula::and([l, r])));
            }
            BinOp::Or => {
                let l = self.lower_bool(lhs, scope)?;
                if l == Formula::True {
                    return Ok(SVal::Bool(l));
                }
                let r = self.under(l.negate()?, |s| s.lower_bool(rhs, scope))?;
                return Ok(SVal::Bool(Formula::or([l, r])));
            }
            BinOp::Coalesce => {
                return match self.lower(lhs, scope)? {
                    SVal::NoneLit => self.lower(rhs, scope),
                    SVal::Opt { some, value } => {
                        let other = self.under(some.negate()?, |s| s.lower(rhs, scope))?;
                        self.ite(&some, *value, other)
                    }
                    present => Ok(present),
                };
            }
            BinOp::Eq | BinOp::Ne => {
                let (l, r) = (self.lower(lhs, scope)?, self.lower(rhs, scope)?);
                let equal = self.equal(&l, &r, term)?;
                return Ok(SVal::Bool(if op == BinOp::Eq {
                    equal
                } else {
                    equal.negate()?
                }));
            }
            _ => {}
        }
        let (a, b) = (self.lower_int(lhs, scope)?, self.lower_int(rhs, scope)?);
        let arithmetic = |s: &mut Self, value: Poly| -> Res<SVal> {
            s.oblige(
                Hazard::Overflow,
                in_int_range(&value)?,
                Some(value.clone()),
                term,
            );
            Ok(SVal::Int(value))
        };
        match op {
            BinOp::Add => arithmetic(self, fit(a.add(&b))?),
            BinOp::Sub => arithmetic(self, fit(a.sub(&b))?),
            BinOp::Mul => arithmetic(self, fit(a.mul(&b))?),
            BinOp::Div | BinOp::Mod => {
                let nonzero = Formula::same(&b, &int(0))?.negate()?;
                self.oblige(Hazard::DivByZero, nonzero, None, term);
                Ok(SVal::Int(if op == BinOp::Div {
                    self.divmod(&a, &b)?.0
                } else {
                    self.modulo(&a, &b)?
                }))
            }
            BinOp::Pow => match (a.as_const(), b.as_const()) {
                (Some(base), Some(exp @ 0..=127)) if base.checked_pow(exp as u32).is_some() => {
                    arithmetic(self, int(base.pow(exp as u32)))
                }
                (_, Some(exp @ 0..=16)) => {
                    let mut value = int(1);
                    for _ in 0..exp {
                        value = fit(value.mul(&a))?;
                    }
                    arithmetic(self, value)
                }
                _ => {
                    let value = self.opaque("**", vec![a, b]);
                    arithmetic(self, value)
                }
            },
            BinOp::Shl => match b.as_const() {
                Some(k @ 0..=62) => arithmetic(self, fit(a.scale(1 << k))?),
                _ => {
                    let value = self.opaque("<<", vec![a, b]);
                    arithmetic(self, value)
                }
            },
            BinOp::Shr => match b.as_const() {
                Some(k @ 0..=62) => Ok(SVal::Int(self.divmod(&a, &int(1 << k))?.0)),
                _ => Ok(SVal::Int(self.opaque(">>", vec![a, b]))),
            },
            BinOp::BitAnd => Ok(SVal::Int(self.bits(Ac::And, vec![a, b])?)),
            BinOp::BitOr => Ok(SVal::Int(self.bits(Ac::Or, vec![a, b])?)),
            BinOp::Xor => Ok(SVal::Int(self.bits(Ac::Xor, vec![a, b])?)),
            BinOp::Lt => Ok(SVal::Bool(Formula::lt(&a, &b)?)),
            BinOp::Le => Ok(SVal::Bool(Formula::le(&a, &b)?)),
            BinOp::Gt => Ok(SVal::Bool(Formula::lt(&b, &a)?)),
            BinOp::Ge => Ok(SVal::Bool(Formula::le(&b, &a)?)),
            BinOp::And | BinOp::Or | BinOp::Coalesce | BinOp::Eq | BinOp::Ne => unreachable!(),
        }
    }

    fn call(&mut self, callee: SVal, args: Vec<SVal>, term: &Term) -> Res<SVal> {
        match callee {
            SVal::Lambda(closure) => {
                if closure.params.len() != args.len() {
                    let (want, got) = (closure.params.len(), args.len());
                    return Err(Error::new(format!(
                        "`{term}` passes {got} arguments to a function of {want}"
                    )));
                }
                let mut scope = closure.scope.clone();
                for (name, value) in closure.params.iter().zip(args) {
                    scope = Some(Rc::new(Frame {
                        name: name.clone(),
                        value,
                        parent: scope,
                    }));
                }
                self.lower(&closure.body, &scope)
            }
            SVal::Named(name) if self.globals.symbols.contains_key(&name) => {
                self.apply(&name, args, term)
            }
            SVal::Named(name) => {
                if name == "some" && args.len() == 1 {
                    let value = args.into_iter().next().unwrap();
                    return Ok(match value {
                        SVal::Opt { .. } | SVal::NoneLit => value,
                        present => SVal::Opt {
                            some: Formula::True,
                            value: Box::new(present),
                        },
                    });
                }
                let mut ints = Vec::with_capacity(args.len());
                for arg in args {
                    match arg {
                        SVal::Int(p) => ints.push(p),
                        other => {
                            return Err(Error::new(format!(
                                "`{term}` passes {} to `{name}`",
                                other.kind()
                            )))
                        }
                    }
                }
                let value = match (name.as_str(), ints.len()) {
                    ("min", 1..) => self.extreme(Ac::Min, ints)?,
                    ("max", 1..) => self.extreme(Ac::Max, ints)?,
                    ("abs", 1) => {
                        let negated = fit(ints[0].neg())?;
                        self.extreme(Ac::Max, vec![ints.pop().unwrap(), negated])?
                    }
                    ("gcd", 1..) => self.bits(Ac::Gcd, ints)?,
                    (_, n) => {
                        return Err(Error::new(format!("`{name}` does not take {n} arguments")))
                    }
                };
                Ok(SVal::Int(value))
            }
            other => Err(Error::new(format!("`{term}` calls {}", other.kind()))),
        }
    }

    /// `a == b`, field by field.
    fn equal(&mut self, a: &SVal, b: &SVal, term: &Term) -> Res<Formula> {
        Ok(match (a, b) {
            (SVal::Int(x), SVal::Int(y)) => Formula::same(x, y)?,
            (SVal::Bool(x), SVal::Bool(y)) => Formula::or([
                Formula::and([x.clone(), y.clone()]),
                Formula::and([x.negate()?, y.negate()?]),
            ]),
            (SVal::Tuple(xs), SVal::Tuple(ys)) if xs.len() == ys.len() => {
                let mut parts = Vec::with_capacity(xs.len());
                for ((_, x), (_, y)) in xs.iter().zip(ys) {
                    parts.push(self.equal(x, y, term)?);
                }
                Formula::and(parts)
            }
            (SVal::NoneLit, SVal::NoneLit) => Formula::True,
            (SVal::NoneLit, SVal::Opt { some, .. }) | (SVal::Opt { some, .. }, SVal::NoneLit) => {
                some.negate()?
            }
            (SVal::Opt { some: s, value: x }, SVal::Opt { some: t, value: y }) => {
                let payload = self.equal(x, y, term)?;
                Formula::or([
                    Formula::and([s.clone(), t.clone(), payload]),
                    Formula::and([s.negate()?, t.negate()?]),
                ])
            }
            (SVal::Opt { some, value }, present) | (present, SVal::Opt { some, value })
                if !matches!(present, SVal::NoneLit) =>
            {
                Formula::and([some.clone(), self.equal(value, present, term)?])
            }
            (SVal::NoneLit, present) | (present, SVal::NoneLit)
                if matches!(present, SVal::Int(_) | SVal::Bool(_) | SVal::Tuple(_)) =>
            {
                Formula::False
            }
            _ => {
                return Err(Error::new(format!(
                    "`{term}` compares {} with {}",
                    a.kind(),
                    b.kind()
                )))
            }
        })
    }

    /// `if cond: then else: other`. Every integer the two sides disagree
    /// on becomes one fresh variable, and one constraint ties them all to
    /// the condition, so a tuple costs a single case split.
    fn ite(&mut self, cond: &Formula, then: SVal, other: SVal) -> Res<SVal> {
        let mut pairs = Vec::new();
        let shape = self.ite_shape(cond, then, other, &mut pairs)?;
        if pairs.is_empty() {
            return Ok(shape.build(&[]));
        }
        let (thens, others): (Vec<Poly>, Vec<Poly>) = pairs.into_iter().unzip();
        let key = (cond.clone(), thens, others);
        if let Some(vars) = self.store.ites.get(&key) {
            return Ok(shape.build(vars));
        }
        let vars: Vec<Var> = (0..key.1.len()).map(|_| self.store.fresh("if")).collect();
        let (mut taken, mut skipped) = (vec![cond.clone()], vec![cond.negate()?]);
        for (i, v) in vars.iter().enumerate() {
            taken.push(Formula::same(&Poly::var(*v), &key.1[i])?);
            skipped.push(Formula::same(&Poly::var(*v), &key.2[i])?);
        }
        self.store.define(
            &vars,
            Formula::or([Formula::and(taken), Formula::and(skipped)]),
        );
        self.store.ites.insert(key, vars.clone());
        Ok(shape.build(&vars))
    }

    fn ite_shape(
        &mut self,
        cond: &Formula,
        then: SVal,
        other: SVal,
        pairs: &mut Vec<(Poly, Poly)>,
    ) -> Res<Shape> {
        Ok(match (then, other) {
            (SVal::Int(x), SVal::Int(y)) if x == y => Shape::Int(x),
            (SVal::Int(x), SVal::Int(y)) => {
                pairs.push((x, y));
                Shape::Slot(pairs.len() - 1)
            }
            (SVal::Bool(x), SVal::Bool(y)) => Shape::Bool(Formula::or([
                Formula::and([cond.clone(), x]),
                Formula::and([cond.negate()?, y]),
            ])),
            (SVal::Tuple(xs), SVal::Tuple(ys)) if xs.len() == ys.len() => {
                let mut items = Vec::with_capacity(xs.len());
                for ((name, x), (_, y)) in xs.into_iter().zip(ys) {
                    items.push((name, self.ite_shape(cond, x, y, pairs)?));
                }
                Shape::Tuple(items)
            }
            (SVal::NoneLit, SVal::NoneLit) => Shape::NoneLit,
            (SVal::Opt { some: s, value: x }, SVal::Opt { some: t, value: y }) => {
                let some = Formula::or([
                    Formula::and([cond.clone(), s]),
                    Formula::and([cond.negate()?, t]),
                ]);
                Shape::Opt(some, Box::new(self.ite_shape(cond, *x, *y, pairs)?))
            }
            (SVal::NoneLit, SVal::Opt { some, value }) => {
                let absent = SVal::Opt {
                    some: Formula::False,
                    value: Box::new(blank(&value)),
                };
                self.ite_shape(cond, absent, SVal::Opt { some, value }, pairs)?
            }
            (SVal::Opt { some, value }, SVal::NoneLit) => {
                let absent = SVal::Opt {
                    some: Formula::False,
                    value: Box::new(blank(&value)),
                };
                self.ite_shape(cond, SVal::Opt { some, value }, absent, pairs)?
            }
            (SVal::Opt { some, value }, present)
                if !matches!(present, SVal::Lambda(_) | SVal::Named(_)) =>
            {
                let wrapped = SVal::Opt {
                    some: Formula::True,
                    value: Box::new(present),
                };
                self.ite_shape(cond, SVal::Opt { some, value }, wrapped, pairs)?
            }
            (present, SVal::Opt { some, value })
                if !matches!(present, SVal::Lambda(_) | SVal::Named(_)) =>
            {
                let wrapped = SVal::Opt {
                    some: Formula::True,
                    value: Box::new(present),
                };
                self.ite_shape(cond, wrapped, SVal::Opt { some, value }, pairs)?
            }
            (SVal::NoneLit, present)
                if matches!(present, SVal::Int(_) | SVal::Bool(_) | SVal::Tuple(_)) =>
            {
                let absent = SVal::Opt {
                    some: Formula::False,
                    value: Box::new(blank(&present)),
                };
                let wrapped = SVal::Opt {
                    some: Formula::True,
                    value: Box::new(present),
                };
                self.ite_shape(cond, absent, wrapped, pairs)?
            }
            (present, SVal::NoneLit)
                if matches!(present, SVal::Int(_) | SVal::Bool(_) | SVal::Tuple(_)) =>
            {
                let absent = SVal::Opt {
                    some: Formula::False,
                    value: Box::new(blank(&present)),
                };
                let wrapped = SVal::Opt {
                    some: Formula::True,
                    value: Box::new(present),
                };
                self.ite_shape(cond, wrapped, absent, pairs)?
            }
            (x, y) => {
                return Err(Error::new(format!(
                    "the branches of an `if` are {} and {}",
                    x.kind(),
                    y.kind()
                )));
            }
        })
    }

    /// `min` or `max` of the arguments.
    fn extreme(&mut self, op: Ac, args: Vec<Poly>) -> Res<Poly> {
        let max = op == Ac::Max;
        // Each argument is split into its variable part and its constant:
        // of `x + 1` and `x + 3` only one can win.
        let mut best: BTreeMap<Poly, i128> = BTreeMap::new();
        let mut work = args;
        let mut spliced = 0;
        while let Some(arg) = work.pop() {
            if spliced < 64 {
                if let Some((v, c, inner)) = self.inner_extreme(&arg, max) {
                    // c * max(xs) + rest is max(c * x + rest) for c > 0,
                    // and min(...) for c < 0.
                    let rest = fit(arg.sub(&fit(Poly::var(v).scale(c))?))?;
                    for x in inner {
                        work.push(fit(x.scale(c).and_then(|scaled| scaled.add(&rest)))?);
                    }
                    spliced += 1;
                    continue;
                }
            }
            let constant = arg.constant_term();
            let form = fit(arg.add_const(-constant))?;
            let slot = best.entry(form).or_insert(constant);
            *slot = if max {
                (*slot).max(constant)
            } else {
                (*slot).min(constant)
            };
        }
        let mut args: Vec<Poly> = Vec::with_capacity(best.len());
        for (form, constant) in best {
            args.push(fit(form.add_const(constant))?);
        }
        if args.len() == 1 {
            return Ok(args.pop().unwrap());
        }
        let key = (op, args);
        if let Some(v) = self.store.ac.get(&key) {
            return Ok(Poly::var(*v));
        }
        let v = self.store.fresh(if max { "max" } else { "min" });
        let value = Poly::var(v);
        let mut bounds = Vec::with_capacity(key.1.len() + 1);
        let mut attained = Vec::with_capacity(key.1.len());
        for arg in &key.1 {
            bounds.push(if max {
                Formula::le(arg, &value)?
            } else {
                Formula::le(&value, arg)?
            });
            attained.push(Formula::same(&value, arg)?);
        }
        bounds.push(Formula::or(attained));
        self.store.define(&[v], Formula::and(bounds));
        self.store.ac_args.insert(v, key.clone());
        self.store.ac.insert(key, v);
        Ok(value)
    }

    /// A `min`/`max` variable inside `arg` that can be merged into an
    /// enclosing `max` (or `min`): its coefficient and its arguments.
    fn inner_extreme(&self, arg: &Poly, max: bool) -> Option<(Var, i128, Vec<Poly>)> {
        for (mono, c) in arg.terms() {
            let [v] = mono.0.as_slice() else { continue };
            let Some((op, inner)) = self.store.ac_args.get(v) else {
                continue;
            };
            let same_direction = match op {
                Ac::Max => (c > 0) == max,
                Ac::Min => (c > 0) != max,
                _ => continue,
            };
            let alone = arg.terms().filter(|(m, _)| m.0.contains(v)).count() == 1;
            if same_direction && alone {
                return Some((*v, c, inner.clone()));
            }
        }
        None
    }

    /// `^`, `&`, `|` and `gcd`: flattened, sorted, with constants folded.
    fn bits(&mut self, op: Ac, args: Vec<Poly>) -> Res<Poly> {
        let identity = match op {
            Ac::And => -1,
            _ => 0,
        };
        let fold = |a: i128, b: i128| match op {
            Ac::Xor => a ^ b,
            Ac::And => a & b,
            Ac::Or => a | b,
            _ => crate::eval::gcd(a.abs(), b.abs()),
        };
        let mut constant = identity;
        let mut counts: BTreeMap<Poly, usize> = BTreeMap::new();
        let mut work = args;
        while let Some(arg) = work.pop() {
            if let Some(c) = arg.as_const() {
                constant = fold(constant, c);
                continue;
            }
            match arg.as_var().and_then(|v| self.store.ac_args.get(&v)) {
                Some((inner_op, inner)) if *inner_op == op => work.extend(inner.iter().cloned()),
                _ => *counts.entry(arg).or_insert(0) += 1,
            }
        }
        // x ^ x is 0; the other operations ignore repeats.
        let mut args: Vec<Poly> = counts
            .into_iter()
            .filter(|(_, n)| op != Ac::Xor || n % 2 == 1)
            .map(|(p, _)| p)
            .collect();
        let absorbed = match op {
            Ac::And => constant == 0,
            Ac::Or => constant == -1,
            Ac::Gcd => constant == 1,
            _ => false,
        };
        if absorbed || args.is_empty() {
            return Ok(int(constant));
        }
        if args.len() == 1 && constant == identity {
            let only = args.pop().unwrap();
            return if op == Ac::Gcd {
                let negated = fit(only.neg())?;
                self.extreme(Ac::Max, vec![only, negated])
            } else {
                Ok(only)
            };
        }
        if constant != identity {
            args.push(int(constant));
        }
        let key = (op, args);
        if let Some(v) = self.store.ac.get(&key) {
            return Ok(Poly::var(*v));
        }
        let name = match op {
            Ac::Xor => "xor",
            Ac::And => "and",
            Ac::Or => "or",
            _ => "gcd",
        };
        let v = self.store.fresh(name);
        let value = Poly::var(v);
        // What interval reasoning needs to know about the result.
        let lemma = if op == Ac::Gcd {
            Formula::le(&int(0), &value)?
        } else {
            let mut negative = Vec::new();
            let mut bounds = vec![Formula::le(&int(0), &value)?];
            let mut sum = Poly::zero();
            for arg in &key.1 {
                negative.push(Formula::lt(arg, &int(0))?);
                sum = fit(sum.add(arg))?;
                match op {
                    Ac::And => bounds.push(Formula::le(&value, arg)?),
                    Ac::Or => bounds.push(Formula::le(arg, &value)?),
                    _ => {}
                }
            }
            if op != Ac::And {
                bounds.push(Formula::le(&value, &sum)?);
            }
            negative.push(Formula::and(bounds));
            Formula::or(negative)
        };
        self.store.define(&[v], lemma);
        self.store.ac_args.insert(v, key.clone());
        self.store.ac.insert(key, v);
        Ok(value)
    }

    /// An operation the solver knows nothing about: equal arguments give
    /// equal results, and the evaluator settles the rest.
    fn opaque(&mut self, op: &'static str, args: Vec<Poly>) -> Poly {
        let key = (op, args);
        if let Some(v) = self.store.opaque.get(&key) {
            return Poly::var(*v);
        }
        let v = self.store.fresh(op);
        self.store.opaque.insert(key, v);
        Poly::var(v)
    }

    /// The Euclidean quotient and remainder of `a` by `b`.
    fn divmod(&mut self, a: &Poly, b: &Poly) -> Res<(Poly, Poly)> {
        match (a.as_const(), b.as_const()) {
            (Some(x), Some(y)) if y != 0 => {
                return Ok((int(x.div_euclid(y)), int(x.rem_euclid(y))))
            }
            (_, Some(1)) => return Ok((a.clone(), int(0))),
            (_, Some(-1)) => return Ok((fit(a.neg())?, int(0))),
            _ => {}
        }
        let key = (a.clone(), b.clone());
        if let Some((q, r)) = self.store.divmods.get(&key) {
            return Ok((Poly::var(*q), Poly::var(*r)));
        }
        let (q, r) = (self.store.fresh("quotient"), self.store.fresh("remainder"));
        let (qp, rp) = (Poly::var(q), Poly::var(r));
        // a == b * q + r with 0 <= r < |b|.
        let split = Formula::same(a, &fit(b.mul(&qp).and_then(|bq| bq.add(&rp)))?)?;
        let nonneg = Formula::le(&int(0), &rp)?;
        let formula = match b.as_const() {
            Some(c) => Formula::and([split, nonneg, Formula::le(&rp, &int(c.abs() - 1))?]),
            None => {
                let below = Formula::or([
                    Formula::and([Formula::le(&int(1), b)?, Formula::lt(&rp, b)?]),
                    Formula::and([Formula::le(b, &int(-1))?, Formula::lt(&rp, &fit(b.neg())?)?]),
                ]);
                // A reminder to try the sign of the quotient: products
                // with a variable divisor are only bounded case by case.
                let sign = Formula::or([
                    Formula::le(&int(1), &qp)?,
                    Formula::same(&qp, &int(0))?,
                    Formula::le(&qp, &int(-1))?,
                ]);
                Formula::or([
                    Formula::same(b, &int(0))?,
                    Formula::and([split, nonneg, below, sign]),
                ])
            }
        };
        self.store.define(&[q, r], formula);
        self.store.divmods.insert(key.clone(), (q, r));
        self.store.remainders.insert(r, key);
        Ok((qp, rp))
    }

    /// `a % b`. The dividend is first reduced modulo `b`: remainders by
    /// the same divisor are replaced by what they are remainders of, so
    /// `((x * y) % m * z) % m` and `(x * ((y * z) % m)) % m` become the
    /// same variable.
    fn modulo(&mut self, a: &Poly, b: &Poly) -> Res<Poly> {
        let mut reduced = a.clone();
        for _ in 0..16 {
            let inner =
                reduced
                    .vars()
                    .into_iter()
                    .find_map(|v| match self.store.remainders.get(&v) {
                        Some((dividend, divisor)) if divisor == b => Some((v, dividend.clone())),
                        _ => None,
                    });
            match inner {
                Some((v, dividend)) => reduced = fit(reduced.substitute(v, &dividend))?,
                None => break,
            }
        }
        if let Some(c) = b.as_const().filter(|c| *c != 0) {
            let terms: Vec<(Mono, i128)> = reduced
                .terms()
                .map(|(m, k)| (m.clone(), k.rem_euclid(c.abs())))
                .collect();
            reduced = fit(Poly::from_terms(terms))?;
        } else if let Some(m) = b.as_var() {
            let terms: Vec<(Mono, i128)> = reduced
                .terms()
                .filter(|(mono, _)| !mono.0.contains(&m))
                .map(|(mono, k)| (mono.clone(), k))
                .collect();
            reduced = fit(Poly::from_terms(terms))?;
        }
        Ok(self.divmod(&reduced, b)?.1)
    }

    /// An element of a declared array or the result of a declared
    /// function: a fresh value of the declared type, equal to every other
    /// use with equal arguments.
    fn apply(&mut self, symbol: &str, args: Vec<SVal>, term: &Term) -> Res<SVal> {
        let mut flat = Vec::new();
        for arg in &args {
            match arg {
                SVal::Int(_) | SVal::Tuple(_) => arg.leaves(&mut flat),
                other => {
                    return Err(Error::new(format!(
                        "`{term}` passes {} to `{symbol}`",
                        other.kind()
                    )))
                }
            }
        }
        if let Some(app) = self
            .store
            .apps
            .iter()
            .find(|app| app.symbol == symbol && app.flat == flat)
        {
            return Ok(app.result.clone());
        }
        let sort = self.globals.symbols[symbol].clone();
        let (result, facts) = self.fresh(&term.to_string(), &sort)?;
        let vars = result.vars();
        self.store.define(&vars, facts);
        let earlier: Vec<App> = self
            .store
            .apps
            .iter()
            .filter(|app| app.symbol == symbol && app.flat.len() == flat.len())
            .cloned()
            .collect();
        for app in earlier {
            let mut differs = Vec::with_capacity(flat.len() * 2 + 1);
            for (x, y) in flat.iter().zip(&app.flat) {
                differs.push(Formula::lt(x, y)?);
                differs.push(Formula::lt(y, x)?);
            }
            differs.push(self.equal(&result, &app.result, term)?);
            let triggers = vec![vars.clone(), app.result.vars()];
            self.store.lazies.push(Lazy {
                triggers,
                formula: Formula::or(differs),
            });
        }
        self.store.apps.push(App {
            symbol: symbol.to_string(),
            args,
            flat,
            result: result.clone(),
        });
        Ok(result)
    }
}

/// The shape of the result of an `if`, with a slot for each fresh variable.
enum Shape {
    Int(Poly),
    Slot(usize),
    Bool(Formula),
    Tuple(Vec<(Option<String>, Shape)>),
    Opt(Formula, Box<Shape>),
    NoneLit,
}

impl Shape {
    fn build(&self, vars: &[Var]) -> SVal {
        match self {
            Shape::Int(p) => SVal::Int(p.clone()),
            Shape::Slot(i) => SVal::Int(Poly::var(vars[*i])),
            Shape::Bool(f) => SVal::Bool(f.clone()),
            Shape::Tuple(items) => SVal::Tuple(
                items
                    .iter()
                    .map(|(name, item)| (name.clone(), item.build(vars)))
                    .collect(),
            ),
            Shape::Opt(some, value) => SVal::Opt {
                some: some.clone(),
                value: Box::new(value.build(vars)),
            },
            Shape::NoneLit => SVal::NoneLit,
        }
    }
}
