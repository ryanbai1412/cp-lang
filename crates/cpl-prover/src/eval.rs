//! A direct evaluator for terms.
//!
//! The solver never has the last word on a failed proof: every
//! counterexample it proposes is run through this evaluator, which shares
//! no code with the symbolic side. A [`crate::Verdict::Refuted`] therefore
//! always carries values for which the facts hold and the goal is false.
//! The evaluator also renders both sides of a broken law and drives the
//! sampling fallback.

use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use crate::term::{BinOp, IntOp, Operands, Sort, Term, UnOp, Value, INT_MAX, INT_MIN};

/// Why an evaluation stopped without a value.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Trap {
    DivByZero,
    UnwrapNone,
    /// A value left `i128`.
    TooLarge,
    /// An operation left `Int`; only reported in strict mode. Holds the
    /// text of the operation.
    Overflow(String),
    /// The term is not well formed: an unbound name, a type error.
    Stuck(String),
}

fn stuck<T>(message: impl Into<String>) -> Result<T, Trap> {
    Err(Trap::Stuck(message.into()))
}

/// What a term evaluates to: a value or something that can be called.
#[derive(Clone)]
enum Rt {
    Val(Value),
    Lambda(Rc<Closure>),
    /// A builtin or a declared function, called by name.
    Named(String),
}

struct Closure {
    params: Vec<String>,
    body: Term,
    scope: Scope,
}

/// Lambda parameters in scope, innermost first.
type Scope = Option<Rc<Frame>>;

struct Frame {
    name: String,
    value: Rt,
    parent: Scope,
}

fn lookup<'a>(scope: &'a Scope, name: &str) -> Option<&'a Rt> {
    let mut frame = scope.as_ref();
    while let Some(f) = frame {
        if f.name == name {
            return Some(&f.value);
        }
        frame = f.parent.as_ref();
    }
    None
}

/// The contents of one declared array or function: argument values to result.
pub type Table = BTreeMap<Vec<Value>, Value>;

/// Values for everything a term can mention.
#[derive(Clone, Default)]
pub struct Env {
    pub vars: HashMap<String, Value>,
    /// Named terms: `op = (a, b) => a + b`.
    pub defs: HashMap<String, Term>,
    /// The result type of each declared array and function.
    pub symbols: HashMap<String, Sort>,
    pub tables: BTreeMap<String, Table>,
    /// When set, an element that no table mentions gets a pseudo-random
    /// value of its type instead of the one closest to zero.
    pub rng: Option<Rng>,
    /// Every table element read so far, in order.
    pub reads: Vec<(String, Vec<Value>)>,
    /// Whether arithmetic that leaves `Int` traps, as it does in a local
    /// build, instead of carrying on with the exact value.
    pub strict: bool,
}

/// xorshift64*: small, fast and reproducible.
#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
    /// Forces every integer to an end of its range.
    pub corner: Option<Corner>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Corner {
    Lowest,
    Highest,
}

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng {
            state: seed.max(1),
            corner: None,
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        self.state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n.max(1)
    }

    /// A value in `lo..=hi`, biased to the ends and to small numbers,
    /// which is where counterexamples live.
    pub fn int_in(&mut self, lo: i128, hi: i128) -> i128 {
        if lo >= hi {
            return lo;
        }
        match self.corner {
            Some(Corner::Lowest) => return lo,
            Some(Corner::Highest) => return hi,
            None => {}
        }
        let clamp = |v: i128| v.clamp(lo, hi);
        let span = (hi - lo) as u128;
        match self.below(16) {
            0..=2 => lo,
            3..=5 => hi,
            6 => clamp(lo.saturating_add(self.below(3) as i128)),
            7 => clamp(hi.saturating_sub(self.below(3) as i128)),
            8..=11 => clamp(self.below(7) as i128 - 3),
            12 | 13 => clamp(self.below(41) as i128 - 20),
            _ => {
                let wide = ((self.next_u64() as u128) << 64) | self.next_u64() as u128;
                lo + (wide % (span + 1)) as i128
            }
        }
    }
}

impl Env {
    pub fn eval(&mut self, term: &Term) -> Result<Value, Trap> {
        match self.rt(term, &None)? {
            Rt::Val(v) => Ok(v),
            _ => stuck(format!("`{term}` is a function, not a value")),
        }
    }

    pub fn eval_bool(&mut self, term: &Term) -> Result<bool, Trap> {
        match self.eval(term)? {
            Value::Bool(b) => Ok(b),
            _ => stuck(format!("`{term}` is not a Bool")),
        }
    }

    pub fn eval_int(&mut self, term: &Term) -> Result<i128, Trap> {
        match self.eval(term)? {
            Value::Int(v) => Ok(v),
            _ => stuck(format!("`{term}` is not an Int")),
        }
    }

    /// Whether `value` is a value of type `sort`.
    pub fn inhabits(&mut self, value: &Value, sort: &Sort) -> Result<bool, Trap> {
        Ok(match (sort, value) {
            (Sort::Int, Value::Int(v)) => (INT_MIN..=INT_MAX).contains(v),
            (Sort::Bool, Value::Bool(_)) => true,
            (Sort::Range { lo, hi, inclusive }, Value::Int(v)) => {
                let (lo, hi) = (self.eval_int(lo)?, self.eval_int(hi)?);
                lo <= *v && if *inclusive { *v <= hi } else { *v < hi }
            }
            (Sort::Tuple(sorts), Value::Tuple(values)) if sorts.len() == values.len() => {
                for ((_, sort), (_, value)) in sorts.iter().zip(values) {
                    if !self.inhabits(value, sort)? {
                        return Ok(false);
                    }
                }
                true
            }
            (Sort::Opt(_), Value::None) => true,
            (Sort::Opt(inner), value) => self.inhabits(value, inner)?,
            _ => false,
        })
    }

    /// A value of type `sort`: the one closest to zero, or a pseudo-random
    /// one when the environment has a generator.
    pub fn inhabitant(&mut self, sort: &Sort) -> Result<Value, Trap> {
        Ok(match sort {
            Sort::Int => Value::Int(self.pick(INT_MIN, INT_MAX)),
            Sort::Bool => Value::Bool(self.rng.as_mut().is_some_and(|rng| rng.below(2) == 1)),
            Sort::Range { lo, hi, inclusive } => {
                let (lo, hi) = (self.eval_int(lo)?, self.eval_int(hi)?);
                let hi = if *inclusive { hi } else { hi - 1 };
                if lo > hi {
                    return stuck(format!("the range `{sort}` is empty"));
                }
                Value::Int(self.pick(lo, hi))
            }
            Sort::Tuple(sorts) => {
                let mut values = Vec::new();
                for (name, sort) in sorts {
                    values.push((name.clone(), self.inhabitant(sort)?));
                }
                Value::Tuple(values)
            }
            Sort::Opt(inner) => match self.rng.as_mut().map(|rng| rng.below(4)) {
                None | Some(0) => Value::None,
                Some(_) => self.inhabitant(inner)?,
            },
            Sort::Named(name) => return stuck(format!("the type `{name}` is not bound")),
        })
    }

    fn pick(&mut self, lo: i128, hi: i128) -> i128 {
        match self.rng.as_mut() {
            Some(rng) => rng.int_in(lo, hi),
            None => 0.clamp(lo, hi),
        }
    }

    fn rt(&mut self, term: &Term, scope: &Scope) -> Result<Rt, Trap> {
        Ok(Rt::Val(match term {
            Term::Int(v) => Value::Int(*v),
            Term::Bool(b) => Value::Bool(*b),
            Term::None => Value::None,
            Term::Var(name) => return self.name(name, scope),
            Term::Unary(op, x) => {
                let x = self.value(x, scope)?;
                match (op, x) {
                    (UnOp::Neg, Value::Int(v)) => {
                        Value::Int(v.checked_neg().ok_or(Trap::TooLarge)?)
                    }
                    (UnOp::Not, Value::Bool(b)) => Value::Bool(!b),
                    (UnOp::Unwrap, Value::None) => return Err(Trap::UnwrapNone),
                    (UnOp::Unwrap, v) => v,
                    _ => return stuck(format!("`{term}` has an operand of the wrong type")),
                }
            }
            Term::Binary(op, lhs, rhs) => self.binary(*op, lhs, rhs, scope, term)?,
            Term::If(cond, then, other) => {
                let branch = if self.truth(cond, scope)? {
                    then
                } else {
                    other
                };
                return self.rt(branch, scope);
            }
            Term::Call(callee, args) => {
                let callee = self.rt(callee, scope)?;
                let mut values = Vec::with_capacity(args.len());
                for arg in args {
                    values.push(self.rt(arg, scope)?);
                }
                return self.call(callee, values, term);
            }
            Term::Index(base, args) => {
                let Term::Var(name) = &**base else {
                    return stuck(format!("`{base}` is not a declared array"));
                };
                let mut values = Vec::with_capacity(args.len());
                for arg in args {
                    values.push(self.value(arg, scope)?);
                }
                self.read(name, values)?
            }
            Term::Tuple(items) => {
                let mut values = Vec::with_capacity(items.len());
                for (name, item) in items {
                    values.push((name.clone(), self.value(item, scope)?));
                }
                Value::Tuple(values)
            }
            Term::Field(base, field) => match self.value(base, scope)? {
                Value::Tuple(items) => {
                    let by_name = items
                        .iter()
                        .position(|(name, _)| name.as_deref() == Some(field));
                    let index = by_name.or_else(|| field.parse().ok().filter(|i| *i < items.len()));
                    match index {
                        Some(i) => items[i].1.clone(),
                        None => return stuck(format!("`{base}` has no field `{field}`")),
                    }
                }
                _ => return stuck(format!("`{base}` has no fields")),
            },
            Term::Lambda(params, body) => {
                let closure = Closure {
                    params: params.clone(),
                    body: (**body).clone(),
                    scope: scope.clone(),
                };
                return Ok(Rt::Lambda(Rc::new(closure)));
            }
            Term::InRange(x, lo, hi, inclusive) => {
                let x = self.int(x, scope)?;
                let (lo, hi) = (self.int(lo, scope)?, self.int(hi, scope)?);
                Value::Bool(lo <= x && if *inclusive { x <= hi } else { x < hi })
            }
        }))
    }

    fn value(&mut self, term: &Term, scope: &Scope) -> Result<Value, Trap> {
        match self.rt(term, scope)? {
            Rt::Val(v) => Ok(v),
            _ => stuck(format!("`{term}` is a function, not a value")),
        }
    }

    fn int(&mut self, term: &Term, scope: &Scope) -> Result<i128, Trap> {
        match self.value(term, scope)? {
            Value::Int(v) => Ok(v),
            _ => stuck(format!("`{term}` is not an Int")),
        }
    }

    fn truth(&mut self, term: &Term, scope: &Scope) -> Result<bool, Trap> {
        match self.value(term, scope)? {
            Value::Bool(b) => Ok(b),
            _ => stuck(format!("`{term}` is not a Bool")),
        }
    }

    fn name(&mut self, name: &str, scope: &Scope) -> Result<Rt, Trap> {
        if let Some(bound) = lookup(scope, name) {
            return Ok(bound.clone());
        }
        if let Some(value) = self.vars.get(name) {
            return Ok(Rt::Val(value.clone()));
        }
        if let Some(def) = self.defs.get(name).cloned() {
            return self.rt(&def, &None);
        }
        if self.symbols.contains_key(name) || is_builtin(name) {
            return Ok(Rt::Named(name.to_string()));
        }
        stuck(format!("`{name}` is not declared"))
    }

    fn binary(
        &mut self,
        op: BinOp,
        lhs: &Term,
        rhs: &Term,
        scope: &Scope,
        term: &Term,
    ) -> Result<Value, Trap> {
        let op = match op.operands() {
            Operands::And => {
                return Ok(Value::Bool(
                    self.truth(lhs, scope)? && self.truth(rhs, scope)?,
                ))
            }
            Operands::Or => {
                return Ok(Value::Bool(
                    self.truth(lhs, scope)? || self.truth(rhs, scope)?,
                ))
            }
            Operands::Coalesce => {
                return match self.value(lhs, scope)? {
                    Value::None => self.value(rhs, scope),
                    v => Ok(v),
                };
            }
            Operands::Eq => {
                return Ok(Value::Bool(same(
                    &self.value(lhs, scope)?,
                    &self.value(rhs, scope)?,
                )))
            }
            Operands::Ne => {
                return Ok(Value::Bool(!same(
                    &self.value(lhs, scope)?,
                    &self.value(rhs, scope)?,
                )))
            }
            Operands::Ints(op) => op,
        };
        let (a, b) = (self.int(lhs, scope)?, self.int(rhs, scope)?);
        let int = |v: Option<i128>| v.map(Value::Int).ok_or(Trap::TooLarge);
        let strict = self.strict;
        // An operation whose exact result may leave `Int`.
        let arithmetic = |v: Option<i128>| match v {
            Some(v) if strict && !(INT_MIN..=INT_MAX).contains(&v) => {
                Err(Trap::Overflow(term.to_string()))
            }
            None if strict => Err(Trap::Overflow(term.to_string())),
            v => int(v),
        };
        match op {
            IntOp::Add => arithmetic(a.checked_add(b)),
            IntOp::Sub => arithmetic(a.checked_sub(b)),
            IntOp::Mul => arithmetic(a.checked_mul(b)),
            IntOp::Div if b == 0 => Err(Trap::DivByZero),
            IntOp::Mod if b == 0 => Err(Trap::DivByZero),
            IntOp::Div => int(a.checked_div_euclid(b)),
            IntOp::Mod => int(a.checked_rem_euclid(b)),
            IntOp::Pow => arithmetic(power(a, b)?),
            IntOp::Shl => arithmetic(shift_left(a, b)?),
            IntOp::Shr => int(Some(shift_right(a, b)?)),
            IntOp::BitAnd => int(Some(a & b)),
            IntOp::BitOr => int(Some(a | b)),
            IntOp::Xor => int(Some(a ^ b)),
            IntOp::Lt => Ok(Value::Bool(a < b)),
            IntOp::Le => Ok(Value::Bool(a <= b)),
            IntOp::Gt => Ok(Value::Bool(a > b)),
            IntOp::Ge => Ok(Value::Bool(a >= b)),
        }
    }

    fn call(&mut self, callee: Rt, args: Vec<Rt>, term: &Term) -> Result<Rt, Trap> {
        match callee {
            Rt::Lambda(closure) => {
                if closure.params.len() != args.len() {
                    let (want, got) = (closure.params.len(), args.len());
                    return stuck(format!(
                        "`{term}` passes {got} arguments to a function of {want}"
                    ));
                }
                let mut scope = closure.scope.clone();
                for (name, value) in closure.params.iter().zip(args) {
                    scope = Some(Rc::new(Frame {
                        name: name.clone(),
                        value,
                        parent: scope,
                    }));
                }
                self.rt(&closure.body, &scope)
            }
            Rt::Named(name) => {
                let mut values = Vec::with_capacity(args.len());
                for arg in args {
                    match arg {
                        Rt::Val(v) => values.push(v),
                        _ => return stuck(format!("`{term}` passes a function to `{name}`")),
                    }
                }
                if self.symbols.contains_key(&name) {
                    return Ok(Rt::Val(self.read(&name, values)?));
                }
                builtin(&name, &values).map(Rt::Val)
            }
            Rt::Val(_) => stuck(format!("`{term}` calls something that is not a function")),
        }
    }

    /// One element of a declared array or function. An element no table
    /// mentions is created on first use, inside its declared type.
    fn read(&mut self, name: &str, args: Vec<Value>) -> Result<Value, Trap> {
        let Some(sort) = self.symbols.get(name).cloned() else {
            return stuck(format!("`{name}` is not a declared array or function"));
        };
        self.reads.push((name.to_string(), args.clone()));
        if let Some(value) = self
            .tables
            .get(name)
            .and_then(|table| table.get(&args))
            .cloned()
        {
            if !self.inhabits(&value, &sort)? {
                return stuck(format!("`{name}` holds a value outside its type"));
            }
            return Ok(value);
        }
        let value = self.inhabitant(&sort)?;
        self.tables
            .entry(name.to_string())
            .or_default()
            .insert(args, value.clone());
        Ok(value)
    }
}

/// Structural equality; tuple field names don't take part.
fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Tuple(xs), Value::Tuple(ys)) => {
            xs.len() == ys.len() && xs.iter().zip(ys).all(|((_, x), (_, y))| same(x, y))
        }
        _ => a == b,
    }
}

pub(crate) fn is_builtin(name: &str) -> bool {
    matches!(name, "min" | "max" | "abs" | "gcd" | "Some")
}

fn builtin(name: &str, args: &[Value]) -> Result<Value, Trap> {
    if name == "Some" && args.len() == 1 {
        return Ok(args[0].clone());
    }
    let mut ints = Vec::with_capacity(args.len());
    for arg in args {
        match arg {
            Value::Int(v) => ints.push(*v),
            _ => return stuck(format!("`{name}` takes Ints")),
        }
    }
    let value = match (name, ints.as_slice()) {
        ("min", [first, rest @ ..]) => rest.iter().fold(*first, |m, v| m.min(*v)),
        ("max", [first, rest @ ..]) => rest.iter().fold(*first, |m, v| m.max(*v)),
        ("abs", [v]) => v.checked_abs().ok_or(Trap::TooLarge)?,
        ("gcd", [first, rest @ ..]) => {
            let mut g = first.checked_abs().ok_or(Trap::TooLarge)?;
            for v in rest {
                g = gcd(g, v.checked_abs().ok_or(Trap::TooLarge)?);
            }
            g
        }
        _ => return stuck(format!("`{name}` does not take {} arguments", args.len())),
    };
    Ok(Value::Int(value))
}

/// The greatest common divisor of two non-negative numbers; `gcd(0, 0) = 0`.
pub(crate) fn gcd(mut a: i128, mut b: i128) -> i128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

fn power(base: i128, exp: i128) -> Result<Option<i128>, Trap> {
    if exp < 0 {
        return stuck("a negative exponent");
    }
    Ok(match (base, u32::try_from(exp)) {
        (_, Ok(exp)) => base.checked_pow(exp),
        (0 | 1, _) => Some(base),
        (-1, _) => Some(if exp % 2 == 0 { 1 } else { -1 }),
        _ => None,
    })
}

fn shift_left(a: i128, k: i128) -> Result<Option<i128>, Trap> {
    if k < 0 {
        return stuck("a negative shift");
    }
    Ok(if k >= 127 {
        (a == 0).then_some(0)
    } else {
        a.checked_mul(1i128 << k)
    })
}

fn shift_right(a: i128, k: i128) -> Result<i128, Trap> {
    if k < 0 {
        return stuck("a negative shift");
    }
    Ok(a >> k.min(127))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn eval(src: &str) -> Result<Value, Trap> {
        Env::default().eval(&parse::term(src).unwrap())
    }

    fn int(src: &str) -> i128 {
        match eval(src) {
            Ok(Value::Int(v)) => v,
            other => panic!("{src} gave {other:?}"),
        }
    }

    #[test]
    fn division_is_euclidean() {
        assert_eq!(int("-7 / 2"), -4);
        assert_eq!(int("-7 % 2"), 1);
        assert_eq!(int("7 / -2"), -3);
        assert_eq!(int("7 % -2"), 1);
        assert_eq!(int("-7 / -2"), 4);
        assert_eq!(int("-7 % -2"), 1);
        assert_eq!(eval("1 / 0"), Err(Trap::DivByZero));
        assert_eq!(eval("1 % 0"), Err(Trap::DivByZero));
    }

    #[test]
    fn operators() {
        assert_eq!(int("2 ** 10 + 3 * 4 - 1"), 1035);
        assert_eq!(int("6 ^ 3"), 5);
        assert_eq!(int("6 & 3 | 8"), 10);
        assert_eq!(int("1 << 40"), 1 << 40);
        assert_eq!(int("-9 >> 1"), -5);
        assert_eq!(int("max(3, 9, 4) - min(3, 9) + abs(-2) + gcd(12, 18)"), 14);
        assert_eq!(eval("1 < 2 and not 2 < 1"), Ok(Value::Bool(true)));
        assert_eq!(eval("3 in 0..3"), Ok(Value::Bool(false)));
        assert_eq!(eval("3 in 0..=3"), Ok(Value::Bool(true)));
    }

    #[test]
    fn lambdas_tuples_and_options() {
        assert_eq!(int("((a, b) => a + b)(2, 3)"), 5);
        assert_eq!(int("(x => y => x - y)(7)(2)"), 5);
        assert_eq!(int("(sum: 4, pre: 9).pre + (1, 2).1"), 11);
        assert_eq!(eval("(sum: 1, pre: 2) == (1, 2)"), Ok(Value::Bool(true)));
        assert_eq!(int("none ?? 4"), 4);
        assert_eq!(int("Some(3)! + (5 ?? 4)"), 8);
        assert_eq!(eval("none!"), Err(Trap::UnwrapNone));
        assert_eq!(eval("Some(3) == none"), Ok(Value::Bool(false)));
    }

    #[test]
    fn short_circuits_skip_traps() {
        assert_eq!(eval("false and 1 / 0 == 0"), Ok(Value::Bool(false)));
        assert_eq!(eval("true or none! == 0"), Ok(Value::Bool(true)));
        assert_eq!(int("if true: 1 else: 1 / 0"), 1);
    }

    #[test]
    fn arrays_are_filled_on_first_read() {
        let mut env = Env::default();
        env.vars.insert("n".to_string(), Value::Int(5));
        env.symbols
            .insert("a".to_string(), parse::sort("2..n").unwrap());
        assert_eq!(
            env.eval(&parse::term("a[0] + a[1]").unwrap()),
            Ok(Value::Int(4))
        );
        env.tables
            .get_mut("a")
            .unwrap()
            .insert(vec![Value::Int(1)], Value::Int(4));
        assert_eq!(
            env.eval(&parse::term("a[0] + a[1]").unwrap()),
            Ok(Value::Int(6))
        );
        assert_eq!(env.reads.len(), 4);
        env.rng = Some(Rng::new(7));
        for i in 2..50 {
            let v = env
                .eval_int(&parse::term(&format!("a[{i}]")).unwrap())
                .unwrap();
            assert!((2..5).contains(&v));
        }
    }

    #[test]
    fn strict_mode_traps_where_a_local_build_does() {
        let term = parse::term("Int.MAX + 1 > 0").unwrap();
        assert_eq!(Env::default().eval(&term), Ok(Value::Bool(true)));
        let mut strict = Env {
            strict: true,
            ..Env::default()
        };
        assert_eq!(
            strict.eval(&term),
            Err(Trap::Overflow("Int.MAX + 1".to_string()))
        );
        assert_eq!(
            strict.eval(&parse::term("-Int.MAX - 0").unwrap()),
            Ok(Value::Int(INT_MIN))
        );
        assert!(matches!(
            strict.eval(&parse::term("3 ** 40").unwrap()),
            Err(Trap::Overflow(_))
        ));
    }

    #[test]
    fn types_are_checked() {
        let mut env = Env::default();
        env.vars.insert("n".to_string(), Value::Int(3));
        let range = parse::sort("0..n").unwrap();
        assert_eq!(env.inhabits(&Value::Int(2), &range), Ok(true));
        assert_eq!(env.inhabits(&Value::Int(3), &range), Ok(false));
        let opt = parse::sort("?(Int, Bool)").unwrap();
        assert_eq!(env.inhabits(&Value::None, &opt), Ok(true));
        assert_eq!(env.inhabits(&Value::Int(1), &opt), Ok(false));
        assert_eq!(
            env.inhabits(&Value::Int(INT_MAX + 1), &Sort::Int),
            Ok(false)
        );
        assert!(matches!(eval("x"), Err(Trap::Stuck(_))));
    }
}
