//! Static check of a fuzzer program: names resolve, types agree, only
//! `var`s are assigned. The generator must always pass it; the shrinker
//! uses it to throw away candidates that are no longer valid programs.

use super::ast::*;
use std::collections::HashMap;

/// fn name -> (parameters, return type)
type Sigs<'a> = HashMap<&'a str, (&'a [(String, Ty)], Ty)>;

pub fn check(p: &Program) -> Result<(), String> {
    let mut sigs: Sigs = HashMap::new();
    for f in &p.funcs {
        let mut c = Checker {
            sigs: &sigs,
            scopes: vec![f
                .params
                .iter()
                .map(|(n, t)| (n.clone(), (*t, false)))
                .collect()],
            ret: Some(f.ret),
            loops: 0,
        };
        c.block(&f.body)?;
        if !always_returns(&f.body) {
            return Err(format!("`{}` can end without a value", f.name));
        }
        // Registered after its body is checked, so a fn can only call
        // earlier ones: no recursion, every call terminates.
        if sigs.insert(&f.name, (&f.params, f.ret)).is_some() {
            return Err(format!("fn `{}` is defined twice", f.name));
        }
    }
    let mut c = Checker {
        sigs: &sigs,
        scopes: vec![HashMap::new()],
        ret: None,
        loops: 0,
    };
    c.block(&p.body)
}

fn always_returns(body: &[Stmt]) -> bool {
    match body.last() {
        Some(Stmt::Return(_) | Stmt::Tail(_)) => true,
        Some(Stmt::If {
            arms,
            els: Some(els),
        }) => arms.iter().all(|(_, b)| always_returns(b)) && always_returns(els),
        _ => false,
    }
}

struct Checker<'a> {
    sigs: &'a Sigs<'a>,
    /// name -> (type, assignable)
    scopes: Vec<HashMap<String, (Ty, bool)>>,
    ret: Option<Ty>,
    loops: u32,
}

impl Checker<'_> {
    fn var(&self, name: &str) -> Result<(Ty, bool), String> {
        self.scopes
            .iter()
            .rev()
            .find_map(|s| s.get(name).copied())
            .ok_or_else(|| format!("unknown name `{name}`"))
    }

    fn list(&self, name: &str) -> Result<bool, String> {
        match self.var(name)? {
            (Ty::List, mutable) => Ok(mutable),
            _ => Err(format!("`{name}` is not a list")),
        }
    }

    fn scoped(&mut self, bind: Option<&str>, body: &[Stmt]) -> Result<(), String> {
        let mut scope = HashMap::new();
        if let Some(name) = bind {
            scope.insert(name.to_string(), (Ty::Int, false));
        }
        self.scopes.push(scope);
        let r = self.block(body);
        self.scopes.pop();
        r
    }

    fn looped(&mut self, bind: Option<&str>, body: &[Stmt]) -> Result<(), String> {
        self.loops += 1;
        let r = self.scoped(bind, body);
        self.loops -= 1;
        r
    }

    fn block(&mut self, stmts: &[Stmt]) -> Result<(), String> {
        if stmts.is_empty() {
            return Err("an empty block".into());
        }
        for (i, s) in stmts.iter().enumerate() {
            let last = i + 1 == stmts.len();
            self.stmt(s, last)?;
        }
        Ok(())
    }

    fn expect(&self, e: &Expr, want: Ty) -> Result<(), String> {
        let got = self.ty(e)?;
        if got == want {
            Ok(())
        } else {
            Err(format!("expected {want:?}, found {got:?}"))
        }
    }

    fn stmt(&mut self, s: &Stmt, last: bool) -> Result<(), String> {
        match s {
            Stmt::Let {
                name,
                ty,
                mutable,
                init,
            } => {
                self.expect(init, *ty)?;
                // No shadowing: every name in a program is distinct, so
                // the output never depends on a backend's scoping rules.
                if self.var(name).is_ok() {
                    return Err(format!("`{name}` is declared twice"));
                }
                self.scopes
                    .last_mut()
                    .unwrap()
                    .insert(name.clone(), (*ty, *mutable));
            }
            Stmt::Assign { name, op, value } => {
                let (ty, mutable) = self.var(name)?;
                if !mutable {
                    return Err(format!("`{name}` is not a `var`"));
                }
                if *op != AssignOp::Set && ty != Ty::Int {
                    return Err(format!("`{}` on a non-Int", op.symbol()));
                }
                self.expect(value, ty)?;
            }
            Stmt::IndexAssign {
                list, index, value, ..
            } => {
                if !self.list(list)? {
                    return Err(format!("`{list}` is not a `var`"));
                }
                self.expect(index, Ty::Int)?;
                self.expect(value, Ty::Int)?;
            }
            Stmt::Push { list, value } => {
                if !self.list(list)? {
                    return Err(format!("`{list}` is not a `var`"));
                }
                self.expect(value, Ty::Int)?;
            }
            Stmt::If { arms, els } => {
                if arms.is_empty() {
                    return Err("`if` without an arm".into());
                }
                for (cond, body) in arms {
                    self.expect(cond, Ty::Bool)?;
                    self.scoped(None, body)?;
                }
                if let Some(body) = els {
                    self.scoped(None, body)?;
                }
            }
            Stmt::ForRange {
                var, lo, hi, body, ..
            } => {
                self.expect(lo, Ty::Int)?;
                self.expect(hi, Ty::Int)?;
                self.looped(Some(var), body)?;
            }
            Stmt::ForIn { var, list, body } => {
                self.list(list)?;
                self.looped(Some(var), body)?;
            }
            Stmt::While { cond, body } => {
                self.expect(cond, Ty::Bool)?;
                self.looped(None, body)?;
            }
            Stmt::Break | Stmt::Continue => {
                if self.loops == 0 {
                    return Err("`break` or `continue` outside a loop".into());
                }
            }
            Stmt::Print(args) => {
                if args.is_empty() {
                    return Err("`print` without arguments".into());
                }
                for a in args {
                    self.ty(a)?;
                }
                if self.ret.is_some() {
                    return Err("`print` inside a fn".into());
                }
            }
            Stmt::Return(e) => match self.ret {
                Some(ret) => self.expect(e, ret)?,
                None => return Err("`return` at the top level".into()),
            },
            Stmt::Tail(e) => match self.ret {
                Some(ret) if last && self.scopes.len() == 1 => self.expect(e, ret)?,
                _ => return Err("a tail expression that is not the end of a fn".into()),
            },
        }
        Ok(())
    }

    pub fn ty(&self, e: &Expr) -> Result<Ty, String> {
        Ok(match e {
            Expr::Int(_) => Ty::Int,
            Expr::Bool(_) => Ty::Bool,
            Expr::Var(n) => self.var(n)?.0,
            Expr::Neg(a) => {
                self.expect(a, Ty::Int)?;
                Ty::Int
            }
            Expr::Not(a) => {
                self.expect(a, Ty::Bool)?;
                Ty::Bool
            }
            Expr::Bin(op, a, b) => match op {
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                    self.expect(a, Ty::Int)?;
                    self.expect(b, Ty::Int)?;
                    Ty::Int
                }
                BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                    self.expect(a, Ty::Int)?;
                    self.expect(b, Ty::Int)?;
                    Ty::Bool
                }
                BinOp::Eq | BinOp::Ne => {
                    let t = self.ty(a)?;
                    if t == Ty::List {
                        return Err("`==` on lists".into());
                    }
                    self.expect(b, t)?;
                    Ty::Bool
                }
                BinOp::And | BinOp::Or => {
                    self.expect(a, Ty::Bool)?;
                    self.expect(b, Ty::Bool)?;
                    Ty::Bool
                }
            },
            Expr::If(c, a, b) => {
                self.expect(c, Ty::Bool)?;
                let t = self.ty(a)?;
                self.expect(b, t)?;
                t
            }
            Expr::Call(name, args) => {
                let (params, ret) = self
                    .sigs
                    .get(name.as_str())
                    .ok_or_else(|| format!("unknown fn `{name}`"))?;
                if params.len() != args.len() {
                    return Err(format!("wrong number of arguments to `{name}`"));
                }
                for ((_, t), a) in params.iter().zip(args) {
                    self.expect(a, *t)?;
                }
                *ret
            }
            Expr::Builtin(b, args) => {
                let want = match b {
                    Builtin::Min | Builtin::Max => 2,
                    Builtin::Abs => 1,
                };
                if args.len() != want {
                    return Err("wrong number of arguments to a builtin".into());
                }
                for a in args {
                    self.expect(a, Ty::Int)?;
                }
                Ty::Int
            }
            Expr::Index(list, i) => {
                self.list(list)?;
                self.expect(i, Ty::Int)?;
                Ty::Int
            }
            Expr::Len(list) => {
                self.list(list)?;
                Ty::Int
            }
            Expr::ListLit(items) => {
                for it in items {
                    self.expect(it, Ty::Int)?;
                }
                Ty::List
            }
        })
    }
}
