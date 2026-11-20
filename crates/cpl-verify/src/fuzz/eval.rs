//! The reference evaluator: the oracle every backend is compared against.
//!
//! It is written for obviousness, not speed, and follows `plan/` exactly:
//! `Int` is 64-bit and overflow is an error, `%` is never negative and
//! `/` is the matching quotient (`a == b * (a / b) + a % b`), `and` / `or`
//! short-circuit, list indexing is checked, `Bool` prints as `YES` / `NO`
//! and a list prints its items separated by spaces.

use super::ast::*;
use std::collections::HashMap;

/// Why a program has no defined output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Trap {
    Overflow,
    DivByZero,
    IndexOutOfRange,
    /// The program ran longer than the evaluator allows.
    StepLimit,
    /// The program is not valid (unknown name, wrong type). The generator
    /// never produces these; the shrinker can, and discards them.
    Invalid(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Int(i64),
    Bool(bool),
    List(Vec<i64>),
}

/// A run that stopped with a trap, and what it had printed by then.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub trap: Trap,
    pub printed: String,
}

enum Flow {
    Next,
    Break,
    Continue,
    Return(Value),
}

pub const DEFAULT_STEP_LIMIT: u64 = 5_000_000;

/// Runs `p` and returns what it prints.
pub fn run(p: &Program) -> Result<String, Failure> {
    run_with_limit(p, DEFAULT_STEP_LIMIT)
}

pub fn run_with_limit(p: &Program, step_limit: u64) -> Result<String, Failure> {
    let mut m = Machine {
        funcs: p.funcs.iter().map(|f| (f.name.as_str(), f)).collect(),
        scopes: vec![HashMap::new()],
        out: String::new(),
        steps: 0,
        step_limit,
    };
    let trap = match m.block(&p.body) {
        Ok(Flow::Next) => return Ok(m.out),
        Ok(_) => invalid("`break`, `continue` or `return` at the top level"),
        Err(trap) => trap,
    };
    Err(Failure {
        trap,
        printed: m.out,
    })
}

/// `%` with a result in `0..|b|`.
pub fn emod(a: i64, b: i64) -> Result<i64, Trap> {
    if b == 0 {
        return Err(Trap::DivByZero);
    }
    // `a % -1` is 0 for every a. `checked_rem_euclid` reports an overflow
    // for the minimum Int, where only the quotient does not fit.
    Ok(a.wrapping_rem_euclid(b))
}

/// The quotient that goes with [`emod`].
pub fn ediv(a: i64, b: i64) -> Result<i64, Trap> {
    if b == 0 {
        return Err(Trap::DivByZero);
    }
    a.checked_div_euclid(b).ok_or(Trap::Overflow)
}

fn invalid(msg: impl Into<String>) -> Trap {
    Trap::Invalid(msg.into())
}

struct Machine<'p> {
    funcs: HashMap<&'p str, &'p Func>,
    scopes: Vec<HashMap<String, Value>>,
    out: String,
    steps: u64,
    step_limit: u64,
}

impl<'p> Machine<'p> {
    fn tick(&mut self) -> Result<(), Trap> {
        self.steps += 1;
        if self.steps > self.step_limit {
            return Err(Trap::StepLimit);
        }
        Ok(())
    }

    fn lookup(&self, name: &str) -> Result<&Value, Trap> {
        self.scopes
            .iter()
            .rev()
            .find_map(|s| s.get(name))
            .ok_or_else(|| invalid(format!("unknown name `{name}`")))
    }

    fn lookup_mut(&mut self, name: &str) -> Result<&mut Value, Trap> {
        self.scopes
            .iter_mut()
            .rev()
            .find_map(|s| s.get_mut(name))
            .ok_or_else(|| invalid(format!("unknown name `{name}`")))
    }

    fn declare(&mut self, name: &str, v: Value) {
        self.scopes.last_mut().unwrap().insert(name.to_string(), v);
    }

    fn scoped(&mut self, stmts: &'p [Stmt]) -> Result<Flow, Trap> {
        self.scopes.push(HashMap::new());
        let r = self.block(stmts);
        self.scopes.pop();
        r
    }

    fn block(&mut self, stmts: &'p [Stmt]) -> Result<Flow, Trap> {
        for s in stmts {
            match self.stmt(s)? {
                Flow::Next => {}
                other => return Ok(other),
            }
        }
        Ok(Flow::Next)
    }

    fn stmt(&mut self, s: &'p Stmt) -> Result<Flow, Trap> {
        self.tick()?;
        match s {
            Stmt::Let { name, ty, init, .. } => {
                let v = self.expr(init)?;
                let ok = matches!(
                    (ty, &v),
                    (Ty::Int, Value::Int(_))
                        | (Ty::Bool, Value::Bool(_))
                        | (Ty::List, Value::List(_))
                );
                if !ok {
                    return Err(invalid(format!(
                        "`{name}` is initialized with the wrong type"
                    )));
                }
                self.declare(name, v);
            }
            Stmt::Assign { name, op, value } => {
                let v = self.expr(value)?;
                let slot = self.lookup_mut(name)?;
                match (op, slot, v) {
                    (AssignOp::Set, Value::Int(s), Value::Int(v)) => *s = v,
                    (AssignOp::Set, Value::Bool(s), Value::Bool(v)) => *s = v,
                    (AssignOp::Set, Value::List(s), Value::List(v)) => *s = v,
                    (AssignOp::Add, Value::Int(s), Value::Int(v)) => {
                        *s = s.checked_add(v).ok_or(Trap::Overflow)?
                    }
                    (AssignOp::Sub, Value::Int(s), Value::Int(v)) => {
                        *s = s.checked_sub(v).ok_or(Trap::Overflow)?
                    }
                    _ => return Err(invalid(format!("bad assignment to `{name}`"))),
                }
            }
            Stmt::IndexAssign {
                list,
                index,
                op,
                value,
            } => {
                let i = self.int(index)?;
                let v = self.int(value)?;
                let Value::List(xs) = self.lookup_mut(list)? else {
                    return Err(invalid(format!("`{list}` is not a list")));
                };
                let slot = usize::try_from(i)
                    .ok()
                    .and_then(|i| xs.get_mut(i))
                    .ok_or(Trap::IndexOutOfRange)?;
                *slot = match op {
                    AssignOp::Set => v,
                    AssignOp::Add => slot.checked_add(v).ok_or(Trap::Overflow)?,
                    AssignOp::Sub => slot.checked_sub(v).ok_or(Trap::Overflow)?,
                };
            }
            Stmt::Push { list, value } => {
                let v = self.int(value)?;
                let Value::List(xs) = self.lookup_mut(list)? else {
                    return Err(invalid(format!("`{list}` is not a list")));
                };
                xs.push(v);
            }
            Stmt::If { arms, els } => {
                for (cond, body) in arms {
                    if self.bool(cond)? {
                        return self.scoped(body);
                    }
                }
                if let Some(body) = els {
                    return self.scoped(body);
                }
            }
            Stmt::ForRange {
                var,
                lo,
                hi,
                inclusive,
                body,
            } => {
                let lo = self.int(lo)?;
                let hi = self.int(hi)?;
                let mut i = lo;
                while if *inclusive { i <= hi } else { i < hi } {
                    self.tick()?;
                    self.scopes
                        .push(HashMap::from([(var.clone(), Value::Int(i))]));
                    let flow = self.block(body);
                    self.scopes.pop();
                    match flow? {
                        Flow::Break => break,
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Next | Flow::Continue => {}
                    }
                    if i == hi {
                        break;
                    }
                    i += 1;
                }
            }
            Stmt::ForIn { var, list, body } => {
                let Value::List(items) = self.lookup(list)?.clone() else {
                    return Err(invalid(format!("`{list}` is not a list")));
                };
                for v in items {
                    self.tick()?;
                    self.scopes
                        .push(HashMap::from([(var.clone(), Value::Int(v))]));
                    let flow = self.block(body);
                    self.scopes.pop();
                    match flow? {
                        Flow::Break => break,
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Next | Flow::Continue => {}
                    }
                }
            }
            Stmt::While { cond, body } => {
                while self.bool(cond)? {
                    self.tick()?;
                    match self.scoped(body)? {
                        Flow::Break => break,
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Next | Flow::Continue => {}
                    }
                }
            }
            Stmt::Break => return Ok(Flow::Break),
            Stmt::Continue => return Ok(Flow::Continue),
            Stmt::Print(args) => {
                let mut parts = Vec::with_capacity(args.len());
                for a in args {
                    parts.push(match self.expr(a)? {
                        Value::Int(v) => v.to_string(),
                        Value::Bool(b) => if b { "YES" } else { "NO" }.to_string(),
                        Value::List(xs) => {
                            xs.iter().map(i64::to_string).collect::<Vec<_>>().join(" ")
                        }
                    });
                }
                self.out.push_str(&parts.join(" "));
                self.out.push('\n');
            }
            Stmt::Return(e) | Stmt::Tail(e) => return Ok(Flow::Return(self.expr(e)?)),
        }
        Ok(Flow::Next)
    }

    fn int(&mut self, e: &'p Expr) -> Result<i64, Trap> {
        match self.expr(e)? {
            Value::Int(v) => Ok(v),
            _ => Err(invalid("expected an Int")),
        }
    }

    fn bool(&mut self, e: &'p Expr) -> Result<bool, Trap> {
        match self.expr(e)? {
            Value::Bool(v) => Ok(v),
            _ => Err(invalid("expected a Bool")),
        }
    }

    fn expr(&mut self, e: &'p Expr) -> Result<Value, Trap> {
        self.tick()?;
        Ok(match e {
            Expr::Int(v) => Value::Int(*v),
            Expr::Bool(b) => Value::Bool(*b),
            Expr::Var(n) => self.lookup(n)?.clone(),
            Expr::Neg(a) => Value::Int(self.int(a)?.checked_neg().ok_or(Trap::Overflow)?),
            Expr::Not(a) => Value::Bool(!self.bool(a)?),
            Expr::Bin(BinOp::And, a, b) => Value::Bool(self.bool(a)? && self.bool(b)?),
            Expr::Bin(BinOp::Or, a, b) => Value::Bool(self.bool(a)? || self.bool(b)?),
            Expr::Bin(op @ (BinOp::Eq | BinOp::Ne), a, b) => {
                let (a, b) = (self.expr(a)?, self.expr(b)?);
                if std::mem::discriminant(&a) != std::mem::discriminant(&b) {
                    return Err(invalid("`==` on different types"));
                }
                Value::Bool((a == b) == (*op == BinOp::Eq))
            }
            Expr::Bin(op, a, b) => {
                let (a, b) = (self.int(a)?, self.int(b)?);
                match op {
                    BinOp::Add => Value::Int(a.checked_add(b).ok_or(Trap::Overflow)?),
                    BinOp::Sub => Value::Int(a.checked_sub(b).ok_or(Trap::Overflow)?),
                    BinOp::Mul => Value::Int(a.checked_mul(b).ok_or(Trap::Overflow)?),
                    BinOp::Div => Value::Int(ediv(a, b)?),
                    BinOp::Mod => Value::Int(emod(a, b)?),
                    BinOp::Lt => Value::Bool(a < b),
                    BinOp::Le => Value::Bool(a <= b),
                    BinOp::Gt => Value::Bool(a > b),
                    BinOp::Ge => Value::Bool(a >= b),
                    BinOp::Eq | BinOp::Ne | BinOp::And | BinOp::Or => unreachable!(),
                }
            }
            Expr::If(c, a, b) => {
                if self.bool(c)? {
                    self.expr(a)?
                } else {
                    self.expr(b)?
                }
            }
            Expr::Call(name, args) => {
                let f = *self
                    .funcs
                    .get(name.as_str())
                    .ok_or_else(|| invalid(format!("unknown fn `{name}`")))?;
                if f.params.len() != args.len() {
                    return Err(invalid(format!("wrong number of arguments to `{name}`")));
                }
                let mut frame = HashMap::new();
                for ((pname, _), a) in f.params.iter().zip(args) {
                    frame.insert(pname.clone(), self.expr(a)?);
                }
                // A fn sees its parameters and nothing from its caller.
                let saved = std::mem::replace(&mut self.scopes, vec![frame]);
                let flow = self.block(&f.body);
                self.scopes = saved;
                match flow? {
                    Flow::Return(v) => v,
                    _ => return Err(invalid(format!("`{name}` ended without a value"))),
                }
            }
            Expr::Builtin(b, args) => {
                let mut vals = Vec::with_capacity(args.len());
                for a in args {
                    vals.push(self.int(a)?);
                }
                Value::Int(match (b, vals.as_slice()) {
                    (Builtin::Min, [x, y]) => *x.min(y),
                    (Builtin::Max, [x, y]) => *x.max(y),
                    (Builtin::Abs, [x]) => x.checked_abs().ok_or(Trap::Overflow)?,
                    _ => return Err(invalid("wrong number of arguments to a builtin")),
                })
            }
            Expr::Index(list, i) => {
                let i = self.int(i)?;
                let Value::List(xs) = self.lookup(list)? else {
                    return Err(invalid(format!("`{list}` is not a list")));
                };
                Value::Int(
                    *usize::try_from(i)
                        .ok()
                        .and_then(|i| xs.get(i))
                        .ok_or(Trap::IndexOutOfRange)?,
                )
            }
            Expr::Len(list) => match self.lookup(list)? {
                Value::List(xs) => Value::Int(xs.len() as i64),
                _ => return Err(invalid(format!("`{list}` is not a list"))),
            },
            Expr::ListLit(items) => {
                let mut xs = Vec::with_capacity(items.len());
                for it in items {
                    xs.push(self.int(it)?);
                }
                Value::List(xs)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn int(v: i64) -> Box<Expr> {
        Box::new(Expr::Int(v))
    }

    fn print_of(e: Expr) -> Result<String, Trap> {
        run(&Program {
            funcs: vec![],
            body: vec![Stmt::Print(vec![e])],
        })
        .map_err(|f| f.trap)
    }

    #[test]
    fn remainder_is_never_negative_and_division_matches_it() {
        // (a, b, a / b, a % b), the table from plan/11 plus negative divisors.
        for (a, b, q, r) in [
            (7, 2, 3, 1),
            (-7, 2, -4, 1),
            (-7, 3, -3, 2),
            (7, -2, -3, 1),
            (-7, -2, 4, 1),
        ] {
            assert_eq!(ediv(a, b), Ok(q), "{a} / {b}");
            assert_eq!(emod(a, b), Ok(r), "{a} % {b}");
            assert_eq!(b * q + r, a);
        }
        assert_eq!(ediv(1, 0), Err(Trap::DivByZero));
        assert_eq!(ediv(i64::MIN, -1), Err(Trap::Overflow));
    }

    #[test]
    fn division_at_the_ends_of_int() {
        // 0 <= a % b < |b| and a == (a / b) * b + a % b, where |b| itself
        // may not fit in an Int.
        for (a, b, q, r) in [
            (i64::MIN, 2, i64::MIN / 2, 0),
            (i64::MIN, 3, -3074457345618258603, 1),
            (i64::MIN, -3, 3074457345618258603, 1),
            (i64::MIN, i64::MAX, -2, i64::MAX - 1),
            (i64::MIN, i64::MIN, 1, 0),
            (i64::MIN + 1, i64::MIN, 1, 1),
            (i64::MAX, i64::MIN, 0, i64::MAX),
            (1, i64::MIN, 0, 1),
            (-1, i64::MIN, 1, i64::MAX),
            (i64::MAX, -1, -i64::MAX, 0),
        ] {
            assert_eq!(ediv(a, b), Ok(q), "{a} / {b}");
            assert_eq!(emod(a, b), Ok(r), "{a} % {b}");
            assert_eq!((b as i128) * (q as i128) + (r as i128), a as i128);
        }
        // Only the quotient of the minimum Int by -1 does not fit; the
        // remainder is 0 like any other `a % -1`.
        assert_eq!(emod(i64::MIN, -1), Ok(0));
        assert_eq!(emod(i64::MIN, 0), Err(Trap::DivByZero));
    }

    #[test]
    fn overflow_is_an_error_not_a_wrap() {
        let e = Expr::Bin(BinOp::Add, int(i64::MAX), int(1));
        assert_eq!(print_of(e), Err(Trap::Overflow));
    }

    #[test]
    fn and_or_do_not_evaluate_the_right_side_when_decided() {
        let div0 = Expr::Bin(
            BinOp::Eq,
            Box::new(Expr::Bin(BinOp::Div, int(1), int(0))),
            int(1),
        );
        let e = Expr::Bin(
            BinOp::And,
            Box::new(Expr::Bool(false)),
            Box::new(div0.clone()),
        );
        assert_eq!(print_of(e).as_deref(), Ok("NO\n"));
        let e = Expr::Bin(BinOp::Or, Box::new(Expr::Bool(true)), Box::new(div0));
        assert_eq!(print_of(e).as_deref(), Ok("YES\n"));
    }

    #[test]
    fn index_out_of_range_is_an_error() {
        let p = Program {
            funcs: vec![],
            body: vec![
                Stmt::Let {
                    name: "xs".into(),
                    ty: Ty::List,
                    mutable: false,
                    init: Expr::ListLit(vec![Expr::Int(1)]),
                },
                Stmt::Print(vec![Expr::Index("xs".into(), int(1))]),
            ],
        };
        assert_eq!(run(&p).unwrap_err().trap, Trap::IndexOutOfRange);
    }

    #[test]
    fn an_endless_loop_hits_the_step_limit() {
        let p = Program {
            funcs: vec![],
            body: vec![Stmt::While {
                cond: Expr::Bool(true),
                body: vec![],
            }],
        };
        assert_eq!(run_with_limit(&p, 1000).unwrap_err().trap, Trap::StepLimit);
    }
}
