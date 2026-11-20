//! Random program generator.
//!
//! Every program it returns passes [`check`](super::check::check) and has
//! a defined output under the reference evaluator, so any backend that
//! disagrees, crashes or trips a sanitizer on it has a bug.
//!
//! The generator stays inside rules that make the output independent of
//! anything `plan/` leaves open:
//!
//! - every name is distinct (no shadowing);
//! - fns are pure and only call earlier fns, so operand evaluation order
//!   cannot be observed and every call terminates;
//! - a divisor is nonzero by construction, the way the checker demands;
//! - a list that a `for` iterates, or whose `len` bounds a range, is not
//!   changed inside that loop;
//! - every `while` has its own counter with a fixed bound.

use super::ast::*;
use super::eval::{self, Trap};
use super::rng::Rng;

#[derive(Debug, Clone)]
pub struct Config {
    pub max_funcs: u64,
    /// Statements in the top-level body, before the closing prints.
    pub max_top_stmts: u64,
    /// Statements in a nested block or fn body.
    pub max_block_stmts: u64,
    /// Nesting of `if` and loops.
    pub max_stmt_depth: u32,
    pub max_expr_depth: u32,
    /// Evaluator steps a generated program may take.
    pub step_limit: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            max_funcs: 3,
            max_top_stmts: 12,
            max_block_stmts: 4,
            max_stmt_depth: 3,
            max_expr_depth: 3,
            step_limit: 200_000,
        }
    }
}

/// A generated program and the output it must produce.
#[derive(Debug, Clone)]
pub struct Case {
    pub seed: u64,
    pub program: Program,
    pub expected: String,
}

/// The program for `seed`. Candidates whose evaluation traps (overflow,
/// index out of range, too many steps) are skipped, deterministically.
pub fn generate(seed: u64, cfg: &Config) -> Case {
    let mut rng = Rng::new(seed);
    loop {
        let program = Gen::new(Rng::new(rng.next_u64()), cfg).program();
        match eval::run_with_limit(&program, cfg.step_limit) {
            Ok(expected) => {
                return Case {
                    seed,
                    program,
                    expected,
                }
            }
            Err(f) => assert!(
                !matches!(f.trap, Trap::Invalid(_)),
                "seed {seed}: generated an invalid program: {:?}",
                f.trap
            ),
        }
    }
}

/// A program that prints what `base` prints and then stops with `trap`.
/// Only the kinds a backend can be asked to detect are supported.
pub fn with_trap(base: &Case, trap: &Trap, rng: &mut Rng) -> Option<Case> {
    let name = "t0".to_string();
    let (setup, bad) = match trap {
        Trap::Overflow => (
            Expr::Int(i64::MAX - rng.range(0, 5)),
            match rng.below(3) {
                0 => bin(BinOp::Add, var(&name), Expr::Int(rng.range(6, 9))),
                1 => bin(BinOp::Mul, var(&name), Expr::Int(rng.range(2, 9))),
                _ => bin(BinOp::Sub, Expr::Int(-rng.range(7, 99)), var(&name)),
            },
        ),
        Trap::DivByZero => (
            Expr::Int(rng.range(-5, 5)),
            bin(
                if rng.chance(1, 2) {
                    BinOp::Div
                } else {
                    BinOp::Mod
                },
                Expr::Int(rng.range(-9, 9)),
                bin(BinOp::Sub, var(&name), var(&name)),
            ),
        ),
        Trap::IndexOutOfRange => {
            let list = "ts0".to_string();
            let len = rng.range(0, 4);
            let index = if rng.chance(1, 2) {
                len + rng.range(0, 3)
            } else {
                -rng.range(1, 3)
            };
            let mut program = base.program.clone();
            program.body.push(Stmt::Let {
                name: list.clone(),
                ty: Ty::List,
                mutable: false,
                init: Expr::ListLit((0..len).map(Expr::Int).collect()),
            });
            program.body.push(Stmt::Let {
                name: name.clone(),
                ty: Ty::Int,
                mutable: false,
                init: Expr::Int(index),
            });
            program
                .body
                .push(Stmt::Print(vec![Expr::Index(list, Box::new(var(&name)))]));
            return Some(Case {
                seed: base.seed,
                program,
                expected: base.expected.clone(),
            });
        }
        Trap::StepLimit | Trap::Invalid(_) => return None,
    };
    let mut program = base.program.clone();
    program.body.push(Stmt::Let {
        name,
        ty: Ty::Int,
        mutable: true,
        init: setup,
    });
    program.body.push(Stmt::Print(vec![bad]));
    Some(Case {
        seed: base.seed,
        program,
        expected: base.expected.clone(),
    })
}

fn var(name: &str) -> Expr {
    Expr::Var(name.to_string())
}

fn bin(op: BinOp, a: Expr, b: Expr) -> Expr {
    Expr::Bin(op, Box::new(a), Box::new(b))
}

#[derive(Debug, Clone)]
struct Var {
    name: String,
    ty: Ty,
    /// The generator may assign it. A `while` counter is a `var` in the
    /// source but is not assignable here, so its loop always ends.
    assignable: bool,
}

struct Sig {
    name: String,
    params: Vec<Ty>,
    ret: Ty,
}

struct Gen<'c> {
    rng: Rng,
    cfg: &'c Config,
    scopes: Vec<Vec<Var>>,
    /// Lists that an enclosing loop reads; they must not change.
    frozen: Vec<String>,
    sigs: Vec<Sig>,
    next_name: u32,
    loops: u32,
    /// `Some(ret)` inside a fn body.
    ret: Option<Ty>,
}

impl<'c> Gen<'c> {
    fn new(rng: Rng, cfg: &'c Config) -> Self {
        Gen {
            rng,
            cfg,
            scopes: vec![vec![]],
            frozen: vec![],
            sigs: vec![],
            next_name: 0,
            loops: 0,
            ret: None,
        }
    }

    fn fresh(&mut self, prefix: &str) -> String {
        self.next_name += 1;
        format!("{prefix}{}", self.next_name)
    }

    fn vars(&self) -> impl Iterator<Item = &Var> {
        self.scopes.iter().flatten()
    }

    fn pick_var(&mut self, want: impl Fn(&Var) -> bool) -> Option<Var> {
        let found: Vec<Var> = self.vars().filter(|v| want(v)).cloned().collect();
        if found.is_empty() {
            None
        } else {
            Some(self.rng.pick(&found).clone())
        }
    }

    fn pick_list(&mut self, to_change: bool) -> Option<String> {
        let frozen = self.frozen.clone();
        self.pick_var(|v| {
            v.ty == Ty::List && (!to_change || (v.assignable && !frozen.contains(&v.name)))
        })
        .map(|v| v.name)
    }

    fn declare(&mut self, name: &str, ty: Ty, assignable: bool) {
        self.scopes.last_mut().unwrap().push(Var {
            name: name.to_string(),
            ty,
            assignable,
        });
    }

    fn program(mut self) -> Program {
        let mut funcs = vec![];
        for _ in 0..self.rng.below(self.cfg.max_funcs + 1) {
            funcs.push(self.func());
        }
        let mut body = vec![];
        // Start with one of each kind of variable so later statements
        // always have something to read and change.
        for ty in [Ty::Int, Ty::Bool, Ty::List] {
            body.push(self.let_stmt(Some(ty)));
        }
        for _ in 0..self.rng.range(2, self.cfg.max_top_stmts as i64) {
            self.stmt(0, &mut body);
        }
        // End by printing every variable, so a wrong value anywhere shows.
        let names: Vec<Expr> = self.scopes[0].iter().map(|v| var(&v.name)).collect();
        for chunk in names.chunks(4) {
            body.push(Stmt::Print(chunk.to_vec()));
        }
        Program { funcs, body }
    }

    fn func(&mut self) -> Func {
        let name = self.fresh("f");
        let mut params = vec![];
        for _ in 0..self.rng.range(1, 3) {
            let ty = *self.rng.pick(&[Ty::Int, Ty::Int, Ty::Bool, Ty::List]);
            params.push((self.fresh("p"), ty));
        }
        let ret = *self.rng.pick(&[Ty::Int, Ty::Int, Ty::Bool]);
        let outer = std::mem::replace(&mut self.scopes, vec![vec![]]);
        for (n, t) in &params {
            self.declare(n, *t, false);
        }
        self.ret = Some(ret);
        let mut body = vec![];
        for _ in 0..self.rng.below(self.cfg.max_block_stmts + 1) {
            self.stmt(1, &mut body);
        }
        let last = self.expr(ret, self.cfg.max_expr_depth);
        body.push(if self.rng.chance(1, 4) {
            Stmt::Return(last)
        } else {
            Stmt::Tail(last)
        });
        self.ret = None;
        self.scopes = outer;
        self.sigs.push(Sig {
            name: name.clone(),
            params: params.iter().map(|(_, t)| *t).collect(),
            ret,
        });
        Func {
            name,
            params,
            ret,
            body,
        }
    }

    fn block(&mut self, depth: u32) -> Vec<Stmt> {
        self.scopes.push(vec![]);
        let mut body = vec![];
        for _ in 0..self.rng.range(1, self.cfg.max_block_stmts as i64) {
            self.stmt(depth, &mut body);
        }
        self.scopes.pop();
        body
    }

    fn loop_body(&mut self, bind: Option<&str>, depth: u32) -> Vec<Stmt> {
        self.scopes.push(vec![]);
        if let Some(name) = bind {
            self.declare(name, Ty::Int, false);
        }
        self.loops += 1;
        let body = self.block(depth);
        self.loops -= 1;
        self.scopes.pop();
        body
    }

    fn let_stmt(&mut self, ty: Option<Ty>) -> Stmt {
        let ty = ty.unwrap_or_else(|| {
            *self
                .rng
                .pick(&[Ty::Int, Ty::Int, Ty::Int, Ty::Bool, Ty::List])
        });
        let init = match ty {
            Ty::List => self.list_lit(),
            _ => self.expr(ty, self.cfg.max_expr_depth),
        };
        let prefix = match ty {
            Ty::Int => "x",
            Ty::Bool => "b",
            Ty::List => "xs",
        };
        let name = self.fresh(prefix);
        let mutable = self.rng.chance(2, 3);
        self.declare(&name, ty, mutable);
        Stmt::Let {
            name,
            ty,
            mutable,
            init,
        }
    }

    /// Never empty, so `i % xs.len` is always a valid index.
    fn list_lit(&mut self) -> Expr {
        let n = self.rng.range(1, 5);
        Expr::ListLit((0..n).map(|_| self.expr(Ty::Int, 1)).collect())
    }

    /// Adds one statement to `out` (a `while` adds its counter too).
    fn stmt(&mut self, depth: u32, out: &mut Vec<Stmt>) {
        let nest = depth < self.cfg.max_stmt_depth;
        loop {
            let stmt = match self.rng.below(16) {
                0..=2 => self.let_stmt(None),
                3..=5 => {
                    let Some(v) = self.pick_var(|v| v.assignable && v.ty != Ty::List) else {
                        continue;
                    };
                    let op = match v.ty {
                        Ty::Int => *self
                            .rng
                            .pick(&[AssignOp::Set, AssignOp::Add, AssignOp::Sub]),
                        _ => AssignOp::Set,
                    };
                    let value = self.expr(v.ty, self.cfg.max_expr_depth);
                    Stmt::Assign {
                        name: v.name,
                        op,
                        value,
                    }
                }
                6 => {
                    let Some(list) = self.pick_list(true) else {
                        continue;
                    };
                    match self.rng.below(4) {
                        0 => Stmt::Assign {
                            name: list,
                            op: AssignOp::Set,
                            value: self.list_lit(),
                        },
                        1 => Stmt::Push {
                            list,
                            value: self.expr(Ty::Int, 2),
                        },
                        _ => Stmt::IndexAssign {
                            index: self.index_into(&list),
                            list,
                            op: *self
                                .rng
                                .pick(&[AssignOp::Set, AssignOp::Add, AssignOp::Sub]),
                            value: self.expr(Ty::Int, 2),
                        },
                    }
                }
                7..=8 if nest => {
                    let mut arms = vec![];
                    for _ in 0..self.rng.range(1, 3) {
                        let cond = self.expr(Ty::Bool, self.cfg.max_expr_depth);
                        arms.push((cond, self.block(depth + 1)));
                    }
                    let els = self.rng.chance(1, 2).then(|| self.block(depth + 1));
                    Stmt::If { arms, els }
                }
                9..=10 if nest => self.for_range(depth),
                11 if nest => {
                    let Some(list) = self.pick_list(false) else {
                        continue;
                    };
                    let var = self.fresh("v");
                    self.frozen.push(list.clone());
                    let body = self.loop_body(Some(&var), depth + 1);
                    self.frozen.pop();
                    Stmt::ForIn { var, list, body }
                }
                12 if nest => return self.while_loop(depth, out),
                13 if self.loops > 0 => {
                    // Guarded, so the rest of the loop body is reachable.
                    let cond = self.expr(Ty::Bool, 2);
                    let jump = if self.rng.chance(1, 2) {
                        Stmt::Break
                    } else {
                        Stmt::Continue
                    };
                    Stmt::If {
                        arms: vec![(cond, vec![jump])],
                        els: None,
                    }
                }
                14 => {
                    if let Some(ret) = self.ret {
                        let cond = self.expr(Ty::Bool, 2);
                        let value = self.expr(ret, 2);
                        Stmt::If {
                            arms: vec![(cond, vec![Stmt::Return(value)])],
                            els: None,
                        }
                    } else {
                        let n = self.rng.range(1, 3);
                        Stmt::Print((0..n).map(|_| self.print_arg()).collect())
                    }
                }
                _ => continue,
            };
            out.push(stmt);
            return;
        }
    }

    fn print_arg(&mut self) -> Expr {
        if self.rng.chance(1, 5) {
            if let Some(list) = self.pick_list(false) {
                return var(&list);
            }
        }
        let ty = *self.rng.pick(&[Ty::Int, Ty::Int, Ty::Bool]);
        self.expr(ty, self.cfg.max_expr_depth)
    }

    fn for_range(&mut self, depth: u32) -> Stmt {
        let var = self.fresh("i");
        let mut froze = false;
        let (lo, hi) = match self.rng.below(4) {
            0 => match self.pick_list(false) {
                Some(list) => {
                    self.frozen.push(list.clone());
                    froze = true;
                    (Expr::Int(0), Expr::Len(list))
                }
                None => (Expr::Int(0), Expr::Int(3)),
            },
            _ => {
                let lo = self.rng.range(-3, 4);
                // Sometimes empty, sometimes a single value.
                (Expr::Int(lo), Expr::Int(lo + self.rng.range(-2, 6)))
            }
        };
        let inclusive = !matches!(hi, Expr::Len(_)) && self.rng.chance(1, 2);
        let body = self.loop_body(Some(&var), depth + 1);
        if froze {
            self.frozen.pop();
        }
        Stmt::ForRange {
            var,
            lo,
            hi,
            inclusive,
            body,
        }
    }

    fn while_loop(&mut self, depth: u32, out: &mut Vec<Stmt>) {
        let counter = self.fresh("w");
        self.declare(&counter, Ty::Int, false);
        let bound = self.rng.range(0, 6);
        let mut cond = bin(BinOp::Lt, var(&counter), Expr::Int(bound));
        if self.rng.chance(1, 3) {
            cond = bin(BinOp::And, cond, self.expr(Ty::Bool, 2));
        }
        // The increment comes first so `continue` cannot skip it.
        let mut body = vec![Stmt::Assign {
            name: counter.clone(),
            op: AssignOp::Add,
            value: Expr::Int(1),
        }];
        body.extend(self.loop_body(None, depth + 1));
        out.push(Stmt::Let {
            name: counter,
            ty: Ty::Int,
            mutable: true,
            init: Expr::Int(0),
        });
        out.push(Stmt::While { cond, body });
    }

    fn index_into(&mut self, list: &str) -> Expr {
        if self.rng.chance(1, 6) {
            // A plain small index: usually valid, and when it is not the
            // evaluator rejects the candidate.
            return Expr::Int(self.rng.range(0, 2));
        }
        bin(
            BinOp::Mod,
            self.expr(Ty::Int, 1),
            Expr::Len(list.to_string()),
        )
    }

    /// Literals stay inside 32 bits: a checker that tracks ranges may
    /// refuse arithmetic on larger constants even where it never runs.
    /// Values near the 64-bit limits come from the trap variants.
    fn int_lit(&mut self) -> Expr {
        Expr::Int(match self.rng.below(20) {
            0 => self.rng.range(-1_000_000_007, 1_000_000_007),
            1 => *self
                .rng
                .pick(&[i32::MAX as i64, i32::MIN as i64, 1 << 30, 46_341, 65_536]),
            2..=7 => self.rng.range(-100, 100),
            _ => self.rng.range(-5, 9),
        })
    }

    /// A value that is never zero, built the way the checker can prove.
    fn divisor(&mut self, depth: u32) -> Expr {
        match self.rng.below(6) {
            0 => {
                let inner = self.expr(Ty::Int, depth);
                Expr::Builtin(
                    Builtin::Max,
                    vec![
                        Expr::Builtin(Builtin::Abs, vec![inner]),
                        Expr::Int(self.rng.range(1, 4)),
                    ],
                )
            }
            1 => {
                let inner = self.expr(Ty::Int, depth);
                Expr::Builtin(
                    Builtin::Min,
                    vec![
                        Expr::Neg(Box::new(Expr::Builtin(Builtin::Abs, vec![inner]))),
                        Expr::Int(-self.rng.range(1, 4)),
                    ],
                )
            }
            2 => Expr::Int(*self.rng.pick(&[1, -1, 2, -2, 1_000_000_007, 998_244_353])),
            _ => {
                let v = self.rng.range(1, 12);
                Expr::Int(if self.rng.chance(1, 4) { -v } else { v })
            }
        }
    }

    fn expr(&mut self, ty: Ty, depth: u32) -> Expr {
        match ty {
            Ty::Int => self.int_expr(depth),
            Ty::Bool => self.bool_expr(depth),
            Ty::List => self.list_lit(),
        }
    }

    fn leaf(&mut self, ty: Ty) -> Expr {
        if self.rng.chance(3, 5) {
            if let Some(v) = self.pick_var(|v| v.ty == ty) {
                return var(&v.name);
            }
        }
        match ty {
            Ty::Int => self.int_lit(),
            _ => Expr::Bool(self.rng.chance(1, 2)),
        }
    }

    fn call(&mut self, ret: Ty, depth: u32) -> Option<Expr> {
        let fits: Vec<usize> = (0..self.sigs.len())
            .filter(|&i| self.sigs[i].ret == ret)
            .collect();
        if fits.is_empty() {
            return None;
        }
        let i = *self.rng.pick(&fits);
        let name = self.sigs[i].name.clone();
        let params = self.sigs[i].params.clone();
        let mut args = vec![];
        for ty in params {
            args.push(match ty {
                Ty::List => match self.pick_list(false) {
                    Some(list) if self.rng.chance(2, 3) => var(&list),
                    _ => self.list_lit(),
                },
                _ => self.expr(ty, depth),
            });
        }
        Some(Expr::Call(name, args))
    }

    fn int_expr(&mut self, depth: u32) -> Expr {
        if depth == 0 {
            return self.leaf(Ty::Int);
        }
        let d = depth - 1;
        loop {
            match self.rng.below(20) {
                0..=3 => return self.leaf(Ty::Int),
                4..=8 => {
                    let op = *self.rng.pick(&[BinOp::Add, BinOp::Sub, BinOp::Mul]);
                    return bin(op, self.int_expr(d), self.int_expr(d));
                }
                9..=11 => {
                    let op = *self.rng.pick(&[BinOp::Div, BinOp::Mod, BinOp::Mod]);
                    let a = self.int_expr(d);
                    return bin(op, a, self.divisor(d));
                }
                12 => return Expr::Neg(Box::new(self.int_expr(d))),
                13 => {
                    let b = *self.rng.pick(&[Builtin::Min, Builtin::Max]);
                    return Expr::Builtin(b, vec![self.int_expr(d), self.int_expr(d)]);
                }
                14 => return Expr::Builtin(Builtin::Abs, vec![self.int_expr(d)]),
                15 => {
                    let c = self.bool_expr(d);
                    return Expr::If(
                        Box::new(c),
                        Box::new(self.int_expr(d)),
                        Box::new(self.int_expr(d)),
                    );
                }
                16..=17 => {
                    if let Some(list) = self.pick_list(false) {
                        return if self.rng.chance(1, 4) {
                            Expr::Len(list)
                        } else {
                            Expr::Index(list.clone(), Box::new(self.index_into(&list)))
                        };
                    }
                }
                _ => {
                    if let Some(e) = self.call(Ty::Int, d) {
                        return e;
                    }
                }
            }
        }
    }

    fn bool_expr(&mut self, depth: u32) -> Expr {
        if depth == 0 {
            return self.leaf(Ty::Bool);
        }
        let d = depth - 1;
        loop {
            match self.rng.below(14) {
                0..=1 => return self.leaf(Ty::Bool),
                2..=6 => {
                    let op = *self.rng.pick(&[
                        BinOp::Lt,
                        BinOp::Le,
                        BinOp::Gt,
                        BinOp::Ge,
                        BinOp::Eq,
                        BinOp::Ne,
                    ]);
                    return bin(op, self.int_expr(d), self.int_expr(d));
                }
                7..=8 => {
                    let op = *self.rng.pick(&[BinOp::And, BinOp::Or]);
                    return bin(op, self.bool_expr(d), self.bool_expr(d));
                }
                9 => return Expr::Not(Box::new(self.bool_expr(d))),
                10 => {
                    let op = *self.rng.pick(&[BinOp::Eq, BinOp::Ne]);
                    return bin(op, self.bool_expr(d), self.bool_expr(d));
                }
                11 => {
                    let c = self.bool_expr(d);
                    return Expr::If(
                        Box::new(c),
                        Box::new(self.bool_expr(d)),
                        Box::new(self.bool_expr(d)),
                    );
                }
                _ => {
                    if let Some(e) = self.call(Ty::Bool, d) {
                        return e;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fuzz::{check, print};

    #[test]
    fn generated_programs_are_valid_and_defined() {
        let cfg = Config::default();
        for seed in 0..300 {
            let case = generate(seed, &cfg);
            check::check(&case.program).unwrap_or_else(|e| {
                panic!("seed {seed}: {e}\n{}", print::to_source(&case.program))
            });
            assert!(!case.expected.is_empty(), "seed {seed} prints nothing");
        }
    }

    #[test]
    fn a_seed_always_gives_the_same_program() {
        let cfg = Config::default();
        assert_eq!(generate(42, &cfg).program, generate(42, &cfg).program);
        assert_ne!(generate(42, &cfg).program, generate(43, &cfg).program);
    }

    #[test]
    fn injected_traps_trap_with_the_requested_kind() {
        let cfg = Config::default();
        for seed in 0..40 {
            let base = generate(seed, &cfg);
            for trap in [Trap::Overflow, Trap::DivByZero, Trap::IndexOutOfRange] {
                let case = with_trap(&base, &trap, &mut Rng::new(seed)).unwrap();
                check::check(&case.program).unwrap();
                let f = eval::run(&case.program).unwrap_err();
                assert_eq!(f.trap, trap, "seed {seed}");
                assert_eq!(f.printed, base.expected, "seed {seed}");
            }
        }
    }
}
