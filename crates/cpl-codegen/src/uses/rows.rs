//! The cells of grids a loop over `slot` reaches by `g[slot, ..]`, and what
//! in its body could move them: the facts `gridrow` walks rows by.

use cpl_ir::walk::{deep_exprs, deep_stmts};
use cpl_ir::{Arg, Block, Cast, Expr, ExprKind, Place, PlaceKind, Stmt, Ty};

/// What a loop body does with grids led by its counter.
pub(crate) struct RowScan {
    /// Each grid reached by `g[slot, ..]`, with its dimensions.
    pub grids: Vec<((bool, usize), usize)>,
    /// The slots the body declares.
    pub declared: Vec<usize>,
    /// The roots the body replaces whole or lends `mut`.
    pub whole: Vec<(bool, usize)>,
    /// Whether the body defines a domain, so a size can change.
    pub sizes: bool,
    /// The fns the body calls.
    pub calls: Vec<usize>,
}

fn bare(e: &Expr) -> &Expr {
    match &e.kind {
        ExprKind::Cast(Cast::Trust, inner) => bare(inner),
        _ => e,
    }
}

pub(crate) fn expr_root(e: &Expr) -> Option<(bool, usize)> {
    match e.kind {
        ExprKind::Local(slot) => Some((false, slot)),
        ExprKind::Global(id) => Some((true, id)),
        _ => None,
    }
}

pub(crate) fn place_root(p: &PlaceKind) -> Option<(bool, usize)> {
    match *p {
        PlaceKind::Local(slot) => Some((false, slot)),
        PlaceKind::Global(id) => Some((true, id)),
        _ => None,
    }
}

/// The dimensions of a grid of more than one whose cells are not `Bool`.
pub(crate) fn grid_dims(ty: &Ty) -> Option<usize> {
    match ty {
        Ty::Array { dims, elem } if dims.len() > 1 && **elem != Ty::Bool => Some(dims.len()),
        _ => None,
    }
}

pub(crate) fn led_by(slot: usize, indices: &[Expr]) -> bool {
    matches!(indices.first().map(|i| &bare(i).kind), Some(ExprKind::Local(s)) if *s == slot)
        && indices[1..]
            .iter()
            .all(crate::uses::effects::writes_nothing)
}

fn root_of(p: &Place) -> ((bool, usize), bool) {
    match &p.kind {
        PlaceKind::Local(slot) => ((false, *slot), true),
        PlaceKind::Global(id) => ((true, *id), true),
        PlaceKind::Index(base, _) | PlaceKind::Field(base, _) | PlaceKind::Some(base) => {
            (root_of(base).0, false)
        }
    }
}

/// The grids `body` reaches by `g[slot, ..]` and what in it could move them.
pub(crate) fn rows_led_by(body: &Block, slot: usize) -> RowScan {
    let mut grids: Vec<((bool, usize), usize)> = Vec::new();
    let mut add = |root: (bool, usize), dims: usize| {
        if !grids.iter().any(|(r, _)| *r == root) {
            grids.push((root, dims));
        }
    };
    let mut declared = Vec::new();
    let mut whole: Vec<(bool, usize)> = Vec::new();
    let mut sizes = false;
    let place_use = |p: &Place, add: &mut dyn FnMut((bool, usize), usize)| {
        if let PlaceKind::Index(base, indices) = &p.kind {
            if let (Some(root), Some(dims)) = (place_root(&base.kind), grid_dims(&base.ty)) {
                if led_by(slot, indices) {
                    add(root, dims);
                }
            }
        }
    };
    deep_stmts(body, &mut |stmt| match stmt {
        Stmt::Let(at, _) | Stmt::For(_, at, ..) => declared.push(*at),
        Stmt::Assign(place, ..) => {
            let (root, top) = root_of(place);
            if top {
                whole.push(root);
            }
            place_use(place, &mut add);
        }
        Stmt::DefDomain { .. } | Stmt::DefDomainFromVal { .. } => sizes = true,
        _ => {}
    });
    let mut calls = Vec::new();
    deep_exprs(body, &mut |e| match &e.kind {
        ExprKind::Index(base, indices) => {
            if let (Some(root), Some(dims)) = (expr_root(base), grid_dims(&base.ty)) {
                if led_by(slot, indices) {
                    add(root, dims);
                }
            }
        }
        ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
            if let ExprKind::Call(id, _) = &e.kind {
                calls.push(*id);
            }
            for a in args {
                if let Arg::Mut(p) = a {
                    let (root, top) = root_of(p);
                    if top {
                        whole.push(root);
                    }
                }
            }
        }
        ExprKind::Mutate(_, place, _) | ExprKind::Sort(place, ..) => {
            let (root, top) = root_of(place);
            if top {
                whole.push(root);
            }
        }
        _ => {}
    });
    RowScan {
        grids,
        declared,
        whole,
        sizes,
        calls,
    }
}
