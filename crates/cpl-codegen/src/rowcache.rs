//! A loop that reaches a row of a nested array by indices it never
//! changes, `ways[mid][lo][rest]` inside `for rest`, checks and loads
//! `ways[mid][lo]` once: the first use takes its address and later ones
//! reuse it. Nothing in the body can move the row or change the sizes
//! checked on the way to it, so every check that is skipped would pass.

use crate::emit::Emitter;
use crate::uses::effects::Touch;
use cpl_ir::walk::{deep_exprs, deep_stmts};
use cpl_ir::{Arg, Block, Expr, ExprKind, Place, PlaceKind, Stmt, Ty};

#[derive(Clone, PartialEq, Debug)]
pub struct Row {
    root: (bool, usize),
    at: Vec<Key>,
}

#[derive(Clone, PartialEq, Debug)]
enum Key {
    Local(usize),
    Global(usize),
    Int(i64),
}

fn single(ty: &Ty) -> bool {
    matches!(ty, Ty::List(_)) || matches!(ty, Ty::Array { dims, .. } if dims.len() == 1)
}

fn is_row(ty: &Ty) -> bool {
    matches!(ty, Ty::List(_) | Ty::Array { .. })
}

fn key(e: &Expr) -> Option<Key> {
    match e.kind {
        ExprKind::Local(slot) => Some(Key::Local(slot)),
        ExprKind::Global(id) => Some(Key::Global(id)),
        ExprKind::Int(v) => Some(Key::Int(v)),
        _ => None,
    }
}

fn expr_path(e: &Expr) -> Option<Row> {
    match &e.kind {
        ExprKind::Local(slot) => Some(Row {
            root: (false, *slot),
            at: Vec::new(),
        }),
        ExprKind::Global(id) => Some(Row {
            root: (true, *id),
            at: Vec::new(),
        }),
        ExprKind::Index(base, indices) if indices.len() == 1 && single(&base.ty) => {
            let mut row = expr_path(base)?;
            row.at.push(key(&indices[0])?);
            Some(row)
        }
        _ => None,
    }
}

fn place_path(p: &Place) -> Option<Row> {
    match &p.kind {
        PlaceKind::Local(slot) => Some(Row {
            root: (false, *slot),
            at: Vec::new(),
        }),
        PlaceKind::Global(id) => Some(Row {
            root: (true, *id),
            at: Vec::new(),
        }),
        PlaceKind::Index(base, indices) if indices.len() == 1 && single(&base.ty) => {
            let mut row = place_path(base)?;
            row.at.push(key(&indices[0])?);
            Some(row)
        }
        _ => None,
    }
}

pub fn of_expr(e: &Expr) -> Option<Row> {
    expr_path(e).filter(|row| !row.at.is_empty() && is_row(&e.ty))
}

pub fn of_place(p: &Place) -> Option<Row> {
    place_path(p).filter(|row| !row.at.is_empty() && is_row(&p.ty))
}

fn depth(p: &Place) -> ((bool, usize), usize) {
    match &p.kind {
        PlaceKind::Local(slot) => ((false, *slot), 0),
        PlaceKind::Global(id) => ((true, *id), 0),
        PlaceKind::Index(base, _) | PlaceKind::Field(base, _) | PlaceKind::Some(base) => {
            let (root, d) = depth(base);
            (root, d + 1)
        }
    }
}

enum Use<'p> {
    Expr(&'p Expr),
    Place(&'p Place),
}

impl<'p> Emitter<'p> {
    /// The rows `for slot in ..: body` can reach once, each named and
    /// with the C++ that reaches it, for `stmt.rs` to declare.
    pub fn loop_rows<'b>(&mut self, slot: usize, body: &'b Block) -> Vec<(Row, String, String)> {
        if self.closure_id.is_some() {
            return Vec::new();
        }
        let mut uses: Vec<(Row, Use<'b>)> = Vec::new();
        let mut add = |row: Row, at: Use<'b>| {
            if !uses.iter().any(|(r, _)| *r == row) {
                uses.push((row, at));
            }
        };
        let mut declared = vec![slot];
        let mut stores: Vec<&'b Place> = Vec::new();
        let mut sizes = false;
        deep_stmts(body, &mut |stmt| match stmt {
            Stmt::Let(at, _) | Stmt::For(_, at, ..) => declared.push(*at),
            Stmt::Assign(place, ..) => {
                stores.push(place);
                if let PlaceKind::Index(base, _) = &place.kind {
                    if let Some(row) = of_place(base) {
                        add(row, Use::Place(base));
                    }
                }
            }
            Stmt::DefDomain { .. } | Stmt::DefDomainFromVal { .. } => sizes = true,
            _ => {}
        });
        let values = Touch::of_block(body).unnamed();
        let mut calls = Vec::new();
        deep_exprs(body, &mut |e| match &e.kind {
            ExprKind::Index(base, _) => {
                if let Some(row) = of_expr(base) {
                    add(row, Use::Expr(base));
                }
            }
            ExprKind::Call(id, args) => {
                calls.push(*id);
                stores.extend(args.iter().filter_map(|a| match a {
                    Arg::Mut(p) => Some(p),
                    Arg::Value(_) => None,
                }));
            }
            ExprKind::CallExpr(_, args) => {
                stores.extend(args.iter().filter_map(|a| match a {
                    Arg::Mut(p) => Some(p),
                    Arg::Value(_) => None,
                }));
            }
            ExprKind::Mutate(_, place, _) | ExprKind::Sort(place, ..) => stores.push(place),
            _ => {}
        });
        if sizes || values || uses.is_empty() {
            return Vec::new();
        }
        let effects = Touch::of_block(body);
        let params = self.fn_id.map_or(0, |f| self.program.fns[f].params.len());
        let summaries = &self.summaries;
        let call_ok = |root: (bool, usize)| {
            calls.iter().all(|id| {
                summaries.get(*id).is_some_and(|s| {
                    !s.opaque && !s.calls_values && !(root.0 && s.writes.contains(&root.1))
                })
            })
        };
        let kept = |row: &Row| {
            let fixed = row.at.iter().all(|k| match k {
                Key::Local(slot) => {
                    !declared.contains(slot) && !effects.writes.contains(&(false, *slot))
                }
                Key::Global(id) => !effects.writes.contains(&(true, *id)) && call_ok((true, *id)),
                Key::Int(_) => true,
            });
            let local = !row.root.0;
            let rebound = local && declared.contains(&row.root.1);
            let reached = local && row.root.1 < params && !calls.is_empty();
            let moved = stores.iter().any(|p| {
                let (root, d) = depth(p);
                root == row.root && d < row.at.len()
            });
            fixed
                && !rebound
                && !reached
                && !moved
                && call_ok(row.root)
                && !self.row_ptrs.iter().any(|(r, ..)| r == row)
        };
        let uses: Vec<(Row, Use<'b>)> = uses.into_iter().filter(|(row, _)| kept(row)).collect();
        let inner = |q: &Row| {
            uses.iter().any(|(p, _)| {
                p.root == q.root && p.at.len() > q.at.len() && p.at.starts_with(&q.at)
            })
        };
        let shallow: Vec<bool> = uses.iter().map(|(q, _)| inner(q)).collect();
        let uses = uses
            .into_iter()
            .zip(shallow)
            .filter(|(_, s)| !s)
            .map(|(u, _)| u);
        uses.filter_map(|(row, at)| {
            let code = match at {
                Use::Expr(e) => self.expr(e).text,
                Use::Place(p) => self.place(p),
            };
            // A row reached without a check is as cheap to reach again.
            code.contains("at(")
                .then(|| (row, self.unique("row"), code))
        })
        .collect()
    }

    /// The C++ for a row a loop being written reaches once.
    pub fn cached_row(&self, row: Option<Row>) -> Option<String> {
        let row = row?;
        let (_, name, _) = self.row_ptrs.iter().find(|(r, ..)| *r == row)?;
        Some(format!("{name}()"))
    }
}
