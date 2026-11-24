//! A loop `for i in lo..hi` that reaches cells of a grid by `g[i, ..]`
//! walks the rows of `g` with a pointer it starts once and moves by a row
//! each round, as a loop over `vector<vector>` loads each row once. With
//! the row found as `g.d + i * n` in every round, g++ sees two rows of
//! one flat buffer as one stride apart and walks them with two pointers;
//! from a pointer each, it reaches both cells by one counter, which is
//! faster. Nothing in the body can move the grid's cells or resize it.

use crate::emit::Emitter;
use crate::names::global_ident;
use crate::uses::effects::Touch;
use crate::uses::rows::{led_by, rows_led_by, RowScan};
use cpl_ir::{Block, Expr, Iter, Ty};

/// The pointer to the row `slot` picks in the grid at `root`.
#[derive(Clone, Debug)]
pub struct GridRow {
    root: (bool, usize),
    slot: usize,
    name: String,
    dims: usize,
}

impl<'p> Emitter<'p> {
    /// The grid rows `for slot in iter: body` walks by pointer: each with
    /// the C++ that declares it before the loop and the step that moves it
    /// a row on with the counter.
    pub fn loop_grid_rows(
        &mut self,
        slot: usize,
        iter: &Iter,
        body: &Block,
    ) -> Vec<(GridRow, String, String)> {
        let Iter::Range {
            lo,
            step: None,
            rev: false,
            closed: false,
            ..
        } = iter
        else {
            return Vec::new();
        };
        if self.closure_id.is_some()
            || !self.is_fixed(lo)
            || self.hoisted.contains(&slot)
            || !self.facts.stable(self.facts_local(slot))
            || matches!(self.locals()[slot].ty, Ty::Enum(_))
        {
            return Vec::new();
        }
        let RowScan {
            grids,
            declared,
            whole,
            sizes,
            calls,
        } = rows_led_by(body, slot);
        if sizes || grids.is_empty() || Touch::of_block(body).unnamed() {
            return Vec::new();
        }
        let params = self.fn_id.map_or(0, |f| self.program.fns[f].params.len());
        let summaries = &self.summaries;
        let call_ok = |root: (bool, usize)| {
            calls.iter().all(|id| {
                summaries.get(*id).is_some_and(|s| {
                    !s.opaque && !s.calls_values && !(root.0 && s.writes.contains(&root.1))
                })
            })
        };
        let kept: Vec<((bool, usize), usize)> = grids
            .into_iter()
            .filter(|(root, _)| {
                let local = !root.0;
                !whole.contains(root)
                    && !(local && declared.contains(&root.1))
                    && !(local && root.1 < params && !calls.is_empty())
                    && call_ok(*root)
            })
            .collect();
        let lo = self.expr(lo).text;
        kept.into_iter()
            .map(|(root, dims)| {
                let g = self.grid_code(root);
                let stride = (1..dims)
                    .map(|k| format!("{g}.n[{k}]"))
                    .collect::<Vec<_>>()
                    .join(" * ");
                let name = self.unique(&format!("{}_row", g.trim_start_matches('_')));
                let start = if lo == "0" {
                    format!("{g}.d.data()")
                } else {
                    format!("{g}.d.data() + (size_t)({lo}) * {stride}")
                };
                let row = GridRow {
                    root,
                    slot,
                    name: name.clone(),
                    dims,
                };
                (
                    row,
                    format!("auto* {name} = {start};"),
                    format!("{name} += {stride}"),
                )
            })
            .collect()
    }

    fn grid_code(&self, (global, at): (bool, usize)) -> String {
        if global {
            global_ident(self.program, at)
        } else {
            self.names[at].clone()
        }
    }

    /// The row pointer, and the grid's C++, of a loop being written over
    /// `i` that reaches the cell `g[i, ..]` of the grid at `root`.
    pub fn grid_row(
        &self,
        root: Option<(bool, usize)>,
        indices: &[Expr],
    ) -> Option<(String, String)> {
        let root = root?;
        let row = self
            .grid_rows
            .iter()
            .rev()
            .find(|r| r.root == root && r.dims == indices.len() && led_by(r.slot, indices))?;
        Some((row.name.clone(), self.grid_code(root)))
    }

    /// [`Emitter::grid_row`]'s cell, with the C++ of every index after the first.
    pub fn grid_row_cell(
        &mut self,
        root: Option<(bool, usize)>,
        indices: &[Expr],
    ) -> Option<String> {
        let row = self.grid_row(root, indices)?;
        let at: Vec<String> = indices.iter().map(|i| self.expr(i).text).collect();
        Some(row_cell(&row, &at))
    }
}

/// The cell at the C++ indices `at` of a grid row `(pointer, grid)`.
pub(crate) fn row_cell((name, g): &(String, String), at: &[String]) -> String {
    let mut tail = String::new();
    for (k, code) in at.iter().enumerate().skip(1) {
        tail = if k == 1 {
            format!("(size_t)({code})")
        } else {
            format!("({tail}) * {g}.n[{k}] + (size_t)({code})")
        };
    }
    format!("{name}[{tail}]")
}
