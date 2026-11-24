//! Rows of a table built in place.

use super::*;

impl<'p> Emitter<'p> {
    /// Emits a `cpl::Row<{cell}, {k}>` field initializer directly on the stack
    /// when `item` is a fixed-size fill, array comprehension, array literal, or
    /// same-field copy, avoiding a temporary heap `std::vector`.
    pub(crate) fn emit_row_init(
        &mut self,
        item: &Expr,
        sid: usize,
        idx: usize,
        cell: &str,
        k: u32,
    ) -> Option<String> {
        let program = self.program;
        if let ExprKind::Builtin(Builtin::ListFilled, fargs) = &item.kind {
            if fargs.len() == 2 {
                let val = self.expr(&fargs[1]).text;
                self.need("row");
                return Some(format!("cpl::Row<{cell}, {k}>::filled({val})"));
            }
        }
        if let ExprKind::Field(base, fidx) = &item.kind {
            if base.ty == Ty::Struct(sid) && *fidx == idx {
                let b = self.expr(base);
                return Some(format!(
                    "{}.{}",
                    b.at(PRIMARY),
                    ident(&program.structs[sid].fields[idx].0)
                ));
            }
        }
        if let ExprKind::Fill(dims, value) = &item.kind {
            if dims.len() == 1 {
                let dim = self.dim(&dims[0]);
                if dim == k.to_string() {
                    self.need("row");
                    return Some(match value {
                        Some(v) => {
                            let val = self.expr(v).text;
                            format!("cpl::Row<{cell}, {k}>::filled({val})")
                        }
                        None => format!("cpl::Row<{cell}, {k}>()"),
                    });
                }
            }
        }
        if let ExprKind::Do(block) = &item.kind {
            let (Stmt::Give(give), rest) = block.stmts.split_last()? else {
                return None;
            };
            let ExprKind::Local(acc) = give.kind else {
                return None;
            };
            if walk::has_give(rest) || self.hoisted.contains(&acc) {
                return None;
            }
            let let_pos = rest.iter().position(|s| {
                matches!(
                    s,
                    Stmt::Let(id, Some(Expr { kind: ExprKind::Fill(dims, _), .. }))
                        if *id == acc && dims.len() == 1
                )
            })?;
            // Only replace `acc` with `cpl::Row` if it is never mutated as a whole `std::vector`.
            let mut safe = true;
            for (pos, stmt) in rest.iter().enumerate() {
                if pos == let_pos {
                    continue;
                }
                walk::deep_stmts(
                    &cpl_ir::Block {
                        stmts: vec![stmt.clone()],
                    },
                    &mut |s| {
                        if let Stmt::Assign(p, ..) = s {
                            if matches!(p.kind, PlaceKind::Local(id) if id == acc) {
                                safe = false;
                            }
                        }
                    },
                );
                walk::deep_exprs(
                    &cpl_ir::Block {
                        stmts: vec![stmt.clone()],
                    },
                    &mut |e| match &e.kind {
                        ExprKind::MutSlice(p, _, _)
                        | ExprKind::Mutate(_, p, _)
                        | ExprKind::Sort(p, _, _)
                            if matches!(p.kind, PlaceKind::Local(id) if id == acc) =>
                        {
                            safe = false;
                        }
                        ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
                            for arg in args {
                                if matches!(arg, Arg::Mut(p) if matches!(p.kind, PlaceKind::Local(id) if id == acc))
                                {
                                    safe = false;
                                }
                            }
                        }
                        _ => {}
                    },
                );
            }
            if !safe {
                return None;
            }
            self.need("row");
            let row_ty = format!("cpl::Row<{cell}, {k}>");
            let code = self.lambda(&row_ty, |this, out| {
                for (pos, stmt) in rest.iter().enumerate() {
                    if pos == let_pos {
                        if let Stmt::Let(
                            id,
                            Some(Expr {
                                kind: ExprKind::Fill(dims, init),
                                ..
                            }),
                        ) = stmt
                        {
                            let dim = this.dim(&dims[0]);
                            if dim != k.to_string() {
                                this.need("fail");
                                out.push_str(&format!(
                                    "{}if ({dim} != {k}) cpl::fail(\"a row of another length than its field has\");\n",
                                    pad(this.indent)
                                ));
                            }
                            let name = this.names[*id].clone();
                            match init {
                                Some(v) => {
                                    let val = this.expr(v).text;
                                    out.push_str(&format!(
                                        "{}{row_ty} {name} = {row_ty}::filled({val});\n",
                                        pad(this.indent)
                                    ));
                                }
                                None => out.push_str(&format!("{}{row_ty} {name};\n", pad(this.indent))),
                            }
                        }
                    } else {
                        this.stmt(stmt, out);
                    }
                }
                let name = this.names[acc].clone();
                out.push_str(&format!("{}return {name};\n", pad(this.indent)));
            });
            return Some(code.text);
        }
        None
    }

    /// True for a field read that is a row kept inside its struct.
    pub(crate) fn is_row(&self, e: &Expr) -> bool {
        matches!(&e.kind, ExprKind::Field(base, index)
            if matches!(base.ty, Ty::Struct(id) if self.rows.contains_key(&(id, *index))))
    }
}
