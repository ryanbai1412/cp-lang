//! A call that hands a fn a table built on the spot, `Prefix([R, C] (i, j)
//! => e)`, where the fn's one read of that parameter is `a[r, c]` inside
//! loops over exactly its keys. The fn gets a variant, `cpl_tab_<name>`,
//! that takes the cell as a callback `(i, j) => e`, so the table is never
//! built. It majorizes: taken only when `e` can neither trap nor be seen and
//! reads nothing the fn writes, so working a cell out when the fn reads it
//! cannot be told from working them all out first, and each cell is worked
//! out at most once.

use std::collections::{BTreeMap, BTreeSet};

use cpl_ir::walk::{children, stmt_blocks, stmt_exprs};
use cpl_ir::{Arg, Block, Expr, ExprKind, Iter, PlaceKind, Program, Stmt, Ty};

use crate::devirt::Devirt;
use crate::emit::{Code, Emitter};
use crate::uses::effects::{pure_call, trap_free_with, Effects};
use crate::uses::places::{each_call, each_closure, each_local, each_part};

/// A table literal `[A, B] (i, j) => value`: the loop variable of each
/// key, what the innermost loop binds first, and the cell's value.
pub(crate) struct Cells<'e> {
    pub vars: Vec<usize>,
    pub pre: &'e [Stmt],
    pub value: &'e Expr,
}

/// `e` as a table literal: `do { let t = fill; for i: for j: ..; t[i, j] = value; give t }`.
pub(crate) fn cells(e: &Expr) -> Option<Cells<'_>> {
    let ExprKind::Do(block) = &e.kind else {
        return None;
    };
    let [Stmt::Let(acc, Some(fill)), Stmt::For(_, v, iter, body), Stmt::Give(give)] =
        block.stmts.as_slice()
    else {
        return None;
    };
    if !matches!(fill.kind, ExprKind::Fill(_, None))
        || !matches!(give.kind, ExprKind::Local(g) if g == *acc)
    {
        return None;
    }
    let mut vars = Vec::new();
    let (mut v, mut iter, mut body) = (*v, iter, body);
    loop {
        if !matches!(
            iter,
            Iter::Range {
                step: None,
                rev: false,
                ..
            }
        ) {
            return None;
        }
        vars.push(v);
        match body.stmts.as_slice() {
            [Stmt::For(_, w, it, inner)] => (v, iter, body) = (*w, it, inner),
            [pre @ .., Stmt::Assign(cell, cpl_ir::AssignOp::Set, value)] => {
                let PlaceKind::Index(base, keys) = &cell.kind else {
                    return None;
                };
                let by_vars = keys.len() == vars.len()
                    && keys
                        .iter()
                        .zip(&vars)
                        .all(|(k, v)| matches!(k.kind, ExprKind::Local(l) if l == *v));
                return (matches!(base.kind, PlaceKind::Local(a) if a == *acc) && by_vars)
                    .then_some(Cells { vars, pre, value });
            }
            _ => return None,
        }
    }
}

/// The table parameter of `fid` that a variant can take as a callback: read
/// once, as `a[k..]`, under loops over exactly those keys, each once.
fn table_param(program: &Program, fid: usize) -> Option<usize> {
    let f = &program.fns[fid];
    if f.params.iter().any(|p| p.mutable)
        || program.judge_fns.contains(&fid)
        || program.entry_fns.contains(&fid)
        || !program.fn_guarded_domains(&f.ret, &f.body).is_empty()
        || f.locals[..f.params.len()]
            .iter()
            .any(|l| l.ty.has_fn(program))
    {
        return None;
    }
    (0..f.params.len()).find(|&slot| {
        if !matches!(f.locals[slot].ty, Ty::Array { .. }) {
            return false;
        }
        let mut uses = 0;
        let mut held = false;
        each_local(&f.body, &mut |l| uses += usize::from(l == slot));
        each_closure(&f.body, &mut |cid| {
            held |= program.closures[cid]
                .captures
                .iter()
                .any(|c| !c.is_global && c.outer == slot)
        });
        let mut read = None;
        in_loops(&f.body, slot, &mut Vec::new(), &mut read);
        uses == 1 && !held && read == Some(true)
    })
}

/// Sets `read` to whether the read of `slot` is `slot[k..]` under loops over
/// exactly `k..`, each over a range.
fn in_loops(block: &Block, slot: usize, loops: &mut Vec<Option<usize>>, read: &mut Option<bool>) {
    for stmt in &block.stmts {
        stmt_exprs(stmt, &mut |e| in_expr(e, slot, loops, read));
        let var = match stmt {
            Stmt::For(_, v, Iter::Range { step: None, .. }, _) => Some(Some(*v)),
            Stmt::For(..) | Stmt::While(..) => Some(None),
            _ => None,
        };
        loops.extend(var);
        stmt_blocks(stmt, &mut |inner| in_loops(inner, slot, loops, read));
        if var.is_some() {
            loops.pop();
        }
    }
}

fn in_expr(e: &Expr, slot: usize, loops: &mut Vec<Option<usize>>, read: &mut Option<bool>) {
    match &e.kind {
        ExprKind::Do(block) => return in_loops(block, slot, loops, read),
        ExprKind::Index(base, keys) if matches!(base.kind, ExprKind::Local(l) if l == slot) => {
            *read = Some(
                keys.len() == loops.len()
                    && keys.iter().zip(loops.iter()).all(
                        |(k, v)| matches!((&k.kind, v), (ExprKind::Local(l), Some(v)) if l == v),
                    ),
            );
        }
        _ => {}
    }
    children(e, &mut |c| in_expr(c, slot, loops, read));
}

/// What deciding a call needs of the program: the fns that are pure and
/// whether checks trap.
#[derive(Clone, Copy)]
struct Facts<'a> {
    program: &'a Program,
    devirt: &'a Devirt<'a>,
    pure: &'a BTreeSet<usize>,
    traps: bool,
}

/// The call `fid(args)` can hand its table in `slot` as a callback: the
/// table is a literal whose cells can neither trap nor be seen and read no
/// global the fn writes, and every other argument is trap-free.
fn callable(at: &Facts, effects: &Effects, slot: usize, args: &[Arg], cells: &Cells) -> bool {
    let Facts {
        program,
        devirt,
        pure,
        traps,
    } = *at;
    let call_ok = |e: &Expr| pure_call(program, devirt, pure, None, traps, e);
    let free = |e: &Expr| trap_free_with(e, traps, &call_ok);
    let mut seen = false;
    let mut look = |e: &Expr| {
        each_part(e, &mut |x| {
            seen |= match &x.kind {
                ExprKind::Read(_)
                | ExprKind::ReadFrom(..)
                | ExprKind::Print(..)
                | ExprKind::Closure(_)
                | ExprKind::Mutate(..) => true,
                ExprKind::Global(g) => effects.writes.contains(g),
                _ => false,
            }
        })
    };
    look(cells.value);
    let pre_ok = cells.pre.iter().all(|s| match s {
        Stmt::Let(_, Some(init)) => {
            look(init);
            free(init)
        }
        _ => false,
    });
    !seen
        && pre_ok
        && !effects.unknown_call
        && free(cells.value)
        && args.iter().enumerate().all(|(s, a)| match a {
            _ if s == slot => true,
            Arg::Value(v) => free(v),
            _ => false,
        })
}

/// Each fn some call hands a table literal it can take as a callback, and
/// that parameter.
pub(crate) fn tables(
    program: &Program,
    devirt: &Devirt,
    pure: &BTreeSet<usize>,
    traps: bool,
    effects: &dyn Fn(usize) -> Effects,
) -> BTreeMap<usize, usize> {
    let mut found = BTreeMap::new();
    let mut visit = |block| {
        each_call(block, &mut |fid, args| {
            if found.contains_key(&fid) {
                return;
            }
            let Some(slot) = table_param(program, fid) else {
                return;
            };
            let Some(Arg::Value(lit)) = args.get(slot) else {
                return;
            };
            if cells(lit).is_some_and(|c| {
                let at = Facts {
                    program,
                    devirt,
                    pure,
                    traps,
                };
                callable(&at, &effects(fid), slot, args, &c)
            }) {
                found.insert(fid, slot);
            }
        });
    };
    program.fns.iter().for_each(|f| visit(&f.body));
    program.closures.iter().for_each(|c| visit(&c.body));
    visit(&program.setup);
    visit(&program.main);
    found
}

impl Emitter<'_> {
    /// The call `id(args)` through its table variant, if it has one.
    pub(crate) fn try_table_call(&mut self, id: usize, args: &[Arg]) -> Option<Code> {
        let &slot = self.tables.get(&id)?;
        let Some(Arg::Value(lit)) = args.get(slot) else {
            return None;
        };
        let cells = cells(lit)?;
        let traps = self.target.checks != crate::Checks::None;
        let at = Facts {
            program: self.program,
            devirt: &self.devirt,
            pure: &self.sink_pure,
            traps,
        };
        if !callable(&at, self.uses.effects(id), slot, args, &cells) {
            return None;
        }
        let mut texts = Vec::with_capacity(args.len());
        for (s, arg) in args.iter().enumerate() {
            let Arg::Value(v) = arg else { return None };
            if s == slot {
                texts.push(self.cell_fn(&cells));
            } else if self.devirt.param_elidable(id, s) && self.devirt.skippable_value(v) {
                texts.push("false".to_string());
            } else if self.program.fns[id].locals[s].ty != Ty::Unit {
                texts.push(self.expr(v).text);
            }
        }
        let name = crate::names::fn_name(&self.program.fns[id].name);
        Some(Code::primary(format!(
            "cpl_tab_{name}({})",
            texts.join(", ")
        )))
    }

    /// The cell of a table literal as a lambda of its keys.
    fn cell_fn(&mut self, cells: &Cells) -> String {
        let keys: Vec<String> = cells
            .vars
            .iter()
            .map(|&v| format!("auto {}", self.names[v]))
            .collect();
        let ty = self.ty(&cells.value.ty);
        let mut body = String::new();
        self.indent += 1;
        self.block(
            &Block {
                stmts: cells.pre.to_vec(),
            },
            &mut body,
        );
        let value = self.expr(cells.value).text;
        self.indent -= 1;
        format!(
            "[&]({}) -> {ty} {{ {}return {value}; }}",
            keys.join(", "),
            body.trim_start()
        )
    }

    /// In a table variant, `a[k..]` of the table it takes as a callback.
    pub(crate) fn tabled_cell(&mut self, table: usize, keys: &[Expr]) -> Code {
        let keys: Vec<String> = keys.iter().map(|k| self.expr(k).text).collect();
        Code::primary(format!("{}({})", self.names[table], keys.join(", ")))
    }
}

/// `signature` with the table parameter `param` taken as a callback.
pub(crate) fn head(signature: &str, name: &str, param: &str) -> Option<String> {
    crate::sink::variant_head(signature, name, param, "cpl_tab_", "CplT", param)
}
