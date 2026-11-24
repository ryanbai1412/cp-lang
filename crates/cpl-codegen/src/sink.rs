//! Reduction sinking: a call that hands a fn a list built on the spot,
//! `sum(net(x) for x in ..)`, where the fn's one use of that parameter is a
//! top-level `for` over it. The fn gets a variant, `cpl_sink_<name>`, that
//! takes the producer as a callback and runs its loop body at each push,
//! so the list is never built. It majorizes: taken only when the fn up to
//! and inside that loop can neither trap, stop early nor be seen, so running
//! it interleaved with the producer cannot be told from running it after.

use std::collections::{BTreeMap, BTreeSet};

use cpl_ir::{Arg, Expr, ExprKind, Iter, Program, Stmt, Ty};

use crate::devirt::Devirt;
use crate::emit::{Code, Emitter};
use crate::fuse::{producer, FusedPush};
use crate::uses::effects::{pure_call, pure_fns, trap_free_stmts, trap_free_with, value_closure};
use crate::walk;

/// What `sinks` finds.
pub(crate) struct Sinks {
    /// Each fn called with an inline list it can sink, and that list parameter.
    pub fns: BTreeMap<usize, usize>,
    /// The `yields` fns among the candidates whose callback form feeds one.
    pub producers: BTreeSet<usize>,
    /// The pure fns, which a closure handed to a sunk call may call.
    pub pure: BTreeSet<usize>,
}

pub(crate) fn sinks(
    program: &Program,
    devirt: &Devirt,
    pushers: &BTreeMap<usize, usize>,
    traps: bool,
) -> Sinks {
    let pure = pure_fns(program, devirt, traps);
    let mut found = BTreeMap::new();
    let mut producers = BTreeSet::new();
    let mut visit = |block| {
        walk::deep_exprs(block, &mut |e: &Expr| {
            let ExprKind::Call(fid, args) = &e.kind else {
                return;
            };
            if let Some(slot) = sink_param(program, devirt, &pure, traps, *fid) {
                if sinkable_args(program, devirt, pushers, &pure, *fid, slot, args) {
                    if let Some(Arg::Value(v)) = args.get(slot) {
                        producers.extend(yields_call(pushers, v));
                    }
                    let (mut f, mut s) = (*fid, slot);
                    found.insert(f, s);
                    while let Some((g, k, _)) = forwards(program, f, s) {
                        if found.insert(g, k).is_some() {
                            break;
                        }
                        (f, s) = (g, k);
                    }
                }
            }
        });
    };
    program.fns.iter().for_each(|f| visit(&f.body));
    program.closures.iter().for_each(|c| visit(&c.body));
    visit(&program.setup);
    visit(&program.main);
    Sinks {
        fns: found,
        producers,
        pure,
    }
}

fn sink_param(
    program: &Program,
    devirt: &Devirt,
    pure: &BTreeSet<usize>,
    traps: bool,
    fid: usize,
) -> Option<usize> {
    sink_param_within(program, devirt, pure, traps, fid, FORWARDS)
}

/// How many fns a list may be handed on through, `sum` to `fold` and the like.
const FORWARDS: usize = 4;

fn sink_param_within(
    program: &Program,
    devirt: &Devirt,
    pure: &BTreeSet<usize>,
    traps: bool,
    fid: usize,
    depth: usize,
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
        let Some(at) = at else {
            return depth > 0
                && forwards(program, fid, slot).is_some_and(|(g, k, args)| {
                    sink_param_within(program, devirt, pure, traps, g, depth - 1) == Some(k)
                        && other_args_ok(program, devirt, pure, g, k, args)
                });
        };
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
        let pure_call = |e: &Expr| pure_call(program, devirt, pure, Some(fid), traps, e);
        // A call of a fn-valued parameter devirt can't pin is as pure as the closure
        // each sunk call passes for it, which `sinkable_args` checks there.
        let call_ok = |e: &Expr| {
            pure_call(e)
                || matches!(&e.kind, ExprKind::CallExpr(callee, args)
                    if matches!(callee.kind, ExprKind::Local(p) if p < f.params.len() && p != slot
                        && devirt.param_target(fid, p).is_none())
                        && args.iter().all(|a| matches!(a,
                            Arg::Value(v) if trap_free_with(v, traps, &pure_call))))
        };
        let mut global = false;
        walk::deep_exprs(&f.body, &mut |e| {
            global |= matches!(e.kind, ExprKind::Global(_))
        });
        matches!(f.locals[slot].ty, Ty::List(_))
            && uses == 1
            && !escapes
            && !global
            && trap_free_stmts(&f.body.stmts[..at], usize::MAX, traps, &call_ok)
            && trap_free_stmts(&body.stmts, usize::MAX, traps, &call_ok)
    })
}

/// The list argument is a producer and every other argument a constant or a
/// pure closure, so nothing the producer writes can reach the fn.
fn sinkable_args(
    program: &Program,
    devirt: &Devirt,
    pushers: &BTreeMap<usize, usize>,
    pure: &BTreeSet<usize>,
    fid: usize,
    slot: usize,
    args: &[Arg],
) -> bool {
    matches!(args.get(slot), Some(Arg::Value(v))
        if producer(v).is_some() || yields_call(pushers, v).is_some())
        && other_args_ok(program, devirt, pure, fid, slot, args)
}

/// A call of a `yields` fn that also exists as `cpl_each_f`: the producer is its callback form.
fn yields_call(pushers: &BTreeMap<usize, usize>, v: &Expr) -> Option<usize> {
    match &v.kind {
        ExprKind::Call(f, args)
            if pushers.contains_key(f) && args.iter().all(|a| matches!(a, Arg::Value(_))) =>
        {
            Some(*f)
        }
        _ => None,
    }
}

/// `fn f(.., xs, ..) = g(.., xs, ..)`, its body only handing the list parameter `slot` on
/// whole: `g`, the slot it takes `xs` in, and the arguments of that call.
fn forwards(program: &Program, fid: usize, slot: usize) -> Option<(usize, usize, &[Arg])> {
    let f = &program.fns[fid];
    let [Stmt::Return(Some(e)) | Stmt::Expr(e)] = f.body.stmts.as_slice() else {
        return None;
    };
    let ExprKind::Call(g, args) = &e.kind else {
        return None;
    };
    let mut uses = 0;
    walk::deep_exprs(&f.body, &mut |e| {
        uses += usize::from(matches!(e.kind, ExprKind::Local(l) if l == slot));
    });
    let k = args.iter().position(
        |a| matches!(a, Arg::Value(v) if matches!(v.kind, ExprKind::Local(l) if l == slot)),
    )?;
    (uses == 1 && program.fns[*g].ret == f.ret).then_some((*g, k, args.as_slice()))
}

/// Every argument but the list in `slot` is a constant or a pure closure.
fn other_args_ok(
    program: &Program,
    devirt: &Devirt,
    pure: &BTreeSet<usize>,
    fid: usize,
    slot: usize,
    args: &[Arg],
) -> bool {
    args.iter().enumerate().all(|(s, arg)| match arg {
        _ if s == slot => true,
        Arg::Value(v) => {
            matches!(
                v.kind,
                ExprKind::Int(_) | ExprKind::Bool(_) | ExprKind::Char(_)
            ) || (devirt.param_elidable(fid, s) && devirt.skippable_value(v))
                || matches!(v.kind, ExprKind::Closure(cid) if value_closure(program, devirt, pure, cid))
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
        if matches!(seq.kind, ExprKind::Local(l) if self.sinking == Some(l)) {
            return self.forward_sink_call(id, slot, args);
        }
        if !sinkable_args(
            self.program,
            &self.devirt,
            &self.pushers,
            &self.sink_pure,
            id,
            slot,
            args,
        ) {
            return None;
        }
        let p = producer(seq);
        if p.as_ref().is_some_and(|p| self.hoisted.contains(p.vec)) {
            return None;
        }
        let mut texts = Vec::with_capacity(args.len());
        for (s, arg) in args.iter().enumerate() {
            let Arg::Value(v) = arg else { return None };
            if s == slot && p.is_none() {
                let put = self.temp();
                let callee = yields_call(&self.pushers, seq)?;
                let call = self.expr(seq).text;
                let name = crate::names::fn_name(&self.program.fns[callee].name);
                let call = (call.strip_prefix("::prog::").unwrap_or(&call))
                    .strip_prefix(&format!("{name}("))
                    .and_then(|rest| rest.strip_suffix(')'))?;
                let sep = if call.is_empty() { "" } else { ", " };
                let x = self.temp();
                texts.push(format!(
                    "[&](auto&& {put}) {{ cpl_each_{name}({call}{sep}[&](auto&& {x}) -> int {{ {put}({x}); return 0; }}); }}"
                ));
            } else if s == slot {
                let Some(p) = &p else { return None };
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

    /// `f(..).len` of a fn with a callback form: counts the values as it
    /// makes them, with no list.
    pub(crate) fn count_pushed(&mut self, call: &Expr) -> Code {
        let text = self.expr(call).text;
        let ExprKind::Call(callee, _) = &call.kind else {
            return Code::new(format!("(ll){text}.size()"), crate::emit::UNARY);
        };
        let name = crate::names::fn_name(&self.program.fns[*callee].name);
        let inner = (text.strip_prefix("::prog::").unwrap_or(&text))
            .strip_prefix(&format!("{name}("))
            .and_then(|rest| rest.strip_suffix(')'));
        let Some(inner) = inner else {
            return Code::new(format!("(ll){text}.size()"), crate::emit::UNARY);
        };
        let sep = if inner.is_empty() { "" } else { ", " };
        let n = self.temp();
        Code::primary(format!(
            "[&]() -> ll {{ ll {n} = 0; cpl_each_{name}({inner}{sep}[&](auto&&) -> int {{ ++{n}; return 0; }}); return {n}; }}()"
        ))
    }

    /// In a sink variant, the call handing the sunk list on: it hands on the callback.
    fn forward_sink_call(&mut self, id: usize, slot: usize, args: &[Arg]) -> Option<Code> {
        if !other_args_ok(self.program, &self.devirt, &self.sink_pure, id, slot, args) {
            return None;
        }
        let mut texts = Vec::with_capacity(args.len());
        for (s, arg) in args.iter().enumerate() {
            let Arg::Value(v) = arg else { return None };
            if s == slot {
                texts.push("cpl_g".to_string());
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

/// For `cpl explain`: each call whose list built on the spot the judge
/// build (`--checks none`) never builds.
pub(crate) fn explained(program: &Program) -> Vec<cpl_diag::Span> {
    let target = crate::Target {
        checks: crate::Checks::None,
        ..crate::Target::default()
    };
    let program = crate::prepared(program, &target);
    let devirt = Devirt::collect(&program);
    let pushers = cpl_ir::pushed::Feeds::candidates(&program).pushers;
    let found = sinks(&program, &devirt, &pushers, false);
    let mut spans = Vec::new();
    let mut visit = |block| {
        walk::deep_exprs(block, &mut |e: &Expr| {
            if let ExprKind::Call(fid, args) = &e.kind {
                if found.fns.get(fid).is_some_and(|&slot| {
                    sinkable_args(&program, &devirt, &pushers, &found.pure, *fid, slot, args)
                }) {
                    spans.push(e.span);
                }
            }
        });
    };
    program.fns.iter().for_each(|f| visit(&f.body));
    program.closures.iter().for_each(|c| visit(&c.body));
    visit(&program.setup);
    visit(&program.main);
    spans
}
