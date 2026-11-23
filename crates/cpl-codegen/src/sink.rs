//! Reduction sinking: a call that hands a fn a list built on the spot,
//! `sum(net(x) for x in ..)`, where the fn's one use of that parameter is a
//! top-level `for` over it. The fn gets a variant, `cpl_sink_<name>`, that
//! takes the producer as a callback and runs its loop body at each push,
//! so the list is never built. It majorizes: taken only when the fn up to
//! and inside that loop can neither trap, stop early nor be seen, so running
//! it interleaved with the producer cannot be told from running it after.

use std::collections::{BTreeMap, BTreeSet};

use cpl_ir::{Arg, Block, Expr, ExprKind, Iter, Program, Stmt, Ty};

use crate::devirt::Devirt;
use crate::emit::{Code, Emitter};
use crate::fuse::{producer, trap_free_stmts, trap_free_with, FusedPush};
use crate::walk;

/// Each fn called with an inline list it can sink, and that list parameter.
pub(crate) fn sinks(program: &Program, devirt: &Devirt) -> BTreeMap<usize, usize> {
    let pure = pure_fns(program, devirt);
    let mut found = BTreeMap::new();
    let mut visit = |block| {
        walk::deep_exprs(block, &mut |e: &Expr| {
            let ExprKind::Call(fid, args) = &e.kind else {
                return;
            };
            if let Some(slot) = sink_param(program, devirt, &pure, *fid) {
                if sinkable_args(program, devirt, *fid, slot, args) {
                    found.insert(*fid, slot);
                }
            }
        });
    };
    program.fns.iter().for_each(|f| visit(&f.body));
    program.closures.iter().for_each(|c| visit(&c.body));
    visit(&program.setup);
    visit(&program.main);
    found
}

fn sink_param(
    program: &Program,
    devirt: &Devirt,
    pure: &BTreeSet<usize>,
    fid: usize,
) -> Option<usize> {
    let f = &program.fns[fid];
    if f.params.iter().any(|p| p.mutable)
        || program.judge_fns.contains(&fid)
        || program.entry_fns.contains(&fid)
        || !program.fn_guarded_domains(&f.ret, &f.body).is_empty()
    {
        return None;
    }
    (0..f.params.len()).find(|&slot| {
        let at = f.body.stmts.iter().position(|s| {
            matches!(s, Stmt::For(_, _, Iter::Seq(seq, false), _)
                if matches!(seq.kind, ExprKind::Local(l) if l == slot))
        });
        let Some(at) = at else { return false };
        let Stmt::For(_, _, _, body) = &f.body.stmts[at] else {
            return false;
        };
        let mut uses = 0;
        walk::deep_exprs(&f.body, &mut |e| {
            uses += usize::from(matches!(e.kind, ExprKind::Local(l) if l == slot));
        });
        let mut escapes = false;
        walk::deep_stmts(body, &mut |s| {
            escapes |= matches!(s, Stmt::Break(_) | Stmt::Continue(_) | Stmt::Return(_));
        });
        let call_ok = |e: &Expr| pure_call(program, devirt, pure, Some(fid), e);
        let mut global = false;
        walk::deep_exprs(&f.body, &mut |e| {
            global |= matches!(e.kind, ExprKind::Global(_))
        });
        matches!(f.locals[slot].ty, Ty::List(_))
            && uses == 1
            && !escapes
            && !global
            && trap_free_stmts(&f.body.stmts[..at], usize::MAX, &call_ok)
            && trap_free_stmts(&body.stmts, usize::MAX, &call_ok)
    })
}

/// The fns that can neither trap, loop forever nor be seen: no mutable
/// parameter, no global read, and a body of trap-free statements whose only
/// calls are to such fns or to closures that are. Built bottom-up, so a
/// recursive fn is never in it.
pub(crate) fn pure_fns(program: &Program, devirt: &Devirt) -> BTreeSet<usize> {
    let mut pure = BTreeSet::new();
    loop {
        let found: Vec<usize> = (0..program.fns.len())
            .filter(|fid| !pure.contains(fid))
            .filter(|&fid| {
                let f = &program.fns[fid];
                !f.params.iter().any(|p| p.mutable)
                    && !program.judge_fns.contains(&fid)
                    && !program.entry_fns.contains(&fid)
                    && pure_body(&f.body, &|e| {
                        pure_call(program, devirt, &pure, Some(fid), e)
                    })
            })
            .collect();
        if found.is_empty() {
            return pure;
        }
        pure.extend(found);
    }
}

/// Trap-free statements up to a `return` of a trap-free value, reading no
/// global; of an `if` decided at compile time only the branch taken counts.
fn pure_body(body: &Block, call_ok: &dyn Fn(&Expr) -> bool) -> bool {
    let mut global = false;
    walk::deep_exprs(body, &mut |e| {
        global |= matches!(e.kind, ExprKind::Global(_))
    });
    !global && pure_stmts(&body.stmts, call_ok)
}

fn pure_stmts(stmts: &[Stmt], call_ok: &dyn Fn(&Expr) -> bool) -> bool {
    let Some((first, rest)) = stmts.split_first() else {
        return true;
    };
    match first {
        Stmt::Return(Some(r)) => trap_free_with(r, call_ok),
        Stmt::If(cond, then, otherwise) if crate::prune::known(cond).is_some() => {
            let taken = if crate::prune::known(cond) == Some(true) {
                then
            } else {
                otherwise
            };
            pure_stmts(&taken.stmts, call_ok)
                && (matches!(taken.stmts.last(), Some(Stmt::Return(_)))
                    || pure_stmts(rest, call_ok))
        }
        _ => {
            trap_free_stmts(std::slice::from_ref(first), usize::MAX, call_ok)
                && pure_stmts(rest, call_ok)
        }
    }
}

/// A call of a fn in `pure`, or of a closure known from `home` to capture
/// nothing and have a pure body, with trap-free arguments.
fn pure_call(
    program: &Program,
    devirt: &Devirt,
    pure: &BTreeSet<usize>,
    home: Option<usize>,
    e: &Expr,
) -> bool {
    let inner = |x: &Expr| pure_call(program, devirt, pure, None, x);
    let (known, args) = match &e.kind {
        ExprKind::Call(fid, args) => (pure.contains(fid), args),
        ExprKind::CallExpr(callee, args) => {
            let known = devirt.target(home, callee).is_some_and(|cid| {
                let c = &program.closures[cid];
                c.muts.is_empty() && c.captures.is_empty() && pure_body(&c.body, &inner)
            });
            (known, args)
        }
        _ => return false,
    };
    let here = |x: &Expr| pure_call(program, devirt, pure, home, x);
    known
        && args
            .iter()
            .all(|a| matches!(a, Arg::Value(v) if trap_free_with(v, &here)))
}

/// The list argument is a producer and every other argument a constant,
/// so nothing the producer writes can reach the fn.
fn sinkable_args(
    program: &Program,
    devirt: &Devirt,
    fid: usize,
    slot: usize,
    args: &[Arg],
) -> bool {
    args.iter().enumerate().all(|(s, arg)| match arg {
        Arg::Value(v) if s == slot => producer(v).is_some(),
        Arg::Value(v) => {
            matches!(
                v.kind,
                ExprKind::Int(_) | ExprKind::Bool(_) | ExprKind::Char(_)
            ) || (devirt.param_elidable(fid, s) && devirt.skippable_value(v))
                || program.fns[fid].locals[s].ty == Ty::Unit && matches!(v.kind, ExprKind::Unit)
        }
        _ => false,
    })
}

impl Emitter<'_> {
    /// The call `id(args)` through its sink variant, if it has one.
    pub(crate) fn try_sink_call(&mut self, id: usize, args: &[Arg]) -> Option<Code> {
        let &slot = self.sinks.get(&id)?;
        let Some(Arg::Value(seq)) = args.get(slot) else {
            return None;
        };
        let p = producer(seq)?;
        if self.hoisted.contains(p.vec)
            || !sinkable_args(self.program, &self.devirt, id, slot, args)
        {
            return None;
        }
        let mut texts = Vec::with_capacity(args.len());
        for (s, arg) in args.iter().enumerate() {
            let Arg::Value(v) = arg else { return None };
            if s == slot {
                let put = self.temp();
                let mut body = String::new();
                self.indent += 1;
                let consumer = FusedPush {
                    var: String::new(),
                    cpp: String::new(),
                    first: Vec::new(),
                    body: Default::default(),
                    sink: Some(put.clone()),
                };
                self.fused_pushes.push((*p.vec, consumer));
                let inside = walk::inside_list(*p.slot, p.iter, p.body, self.locals());
                let before = self.inside.len();
                self.inside
                    .extend(inside.map(|(l, i)| {
                        crate::bounds::Inside::List((false, l, None), i, None, 0, 0)
                    }));
                self.for_stmt(*p.id, *p.slot, p.iter, p.body, &mut body);
                self.inside.truncate(before);
                self.fused_pushes.pop();
                self.indent -= 1;
                let close = crate::emit::pad(self.indent);
                texts.push(format!("[&](auto&& {put}) {{\n{body}{close}}}"));
            } else if self.devirt.param_elidable(id, s) && self.devirt.skippable_value(v) {
                texts.push("false".to_string());
            } else if self.program.fns[id].locals[s].ty != Ty::Unit {
                texts.push(self.expr(v).text);
            }
        }
        let name = crate::names::fn_name(&self.program.fns[id].name);
        Some(Code::primary(format!(
            "cpl_sink_{name}({})",
            texts.join(", ")
        )))
    }

    /// In a sink variant, the loop over the sunk list, as a callback.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn sink_loop(
        &mut self,
        id: cpl_ir::LoopId,
        seq: &Expr,
        var: &str,
        cpp: &str,
        first: &[String],
        body: &cpl_ir::Block,
        out: &mut String,
    ) -> bool {
        if !matches!(seq.kind, ExprKind::Local(l) if self.sinking == Some(l)) {
            return false;
        }
        self.line(out, &format!("cpl_g([&]({cpp} {var}) {{"));
        self.loop_body(id, first, body, out);
        out.truncate(out.trim_end_matches('\n').len());
        out.push_str(");\n");
        true
    }
}

/// `signature` with the list parameter `param` replaced by the callback.
pub(crate) fn head(signature: &str, name: &str, param: &str) -> Option<String> {
    let name = crate::names::fn_name(name);
    let (before, params) = signature.split_once(&format!(" {name}("))?;
    let params = params.strip_suffix(')')?;
    let mut parts = Vec::new();
    let (mut depth, mut start) = (0i32, 0);
    for (i, c) in params.char_indices() {
        match c {
            '<' | '(' | '[' => depth += 1,
            '>' | ')' | ']' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(params[start..i].trim().to_string());
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(params[start..].trim().to_string());
    let at = parts
        .iter()
        .position(|p| p.ends_with(&format!(" {param}")) || p.ends_with(&format!("&{param}")))?;
    parts[at] = "CplG&& cpl_g".to_string();
    let (tparams, ret) = match before
        .strip_prefix("template <")
        .and_then(|t| t.split_once(">\n"))
    {
        Some((t, ret)) => (format!("{t}, "), ret),
        None => (String::new(), before),
    };
    Some(format!(
        "template <{tparams}class CplG>\n{ret} cpl_sink_{name}({})",
        parts.join(", ")
    ))
}
