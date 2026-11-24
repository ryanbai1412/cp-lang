//! Sampled laws (R09.3.9): a stress build checks the laws an operation
//! value's type promises, on the values of each sequence it is handed with.

use super::*;

/// What a stress build adds to the program.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::lower) enum Stress {
    Off,
    /// Read-only `self` methods undo `@cache` writes at exit.
    ColdCache,
    /// A call handing a sequence an operation value checks the value's laws first.
    Laws,
}

/// Lowers a file for `cpl stress`: a call handing a sequence an operation
/// value first checks the laws its type and its traits declare on the sequence's values.
pub fn lower_sampling_laws(analysis: &Analysis) -> Result<Program, Vec<Diagnostic>> {
    super::super::lower_entry_with_cold(analysis, None, false, Stress::Laws)
}

impl<'a> Lowerer<'a> {
    /// The laws of struct `sid`: its own, then those of each trait it
    /// implements (`impl Commutative<Int> for Sub` promises `comm`).
    pub(in crate::lower) fn struct_laws(&self, sid: StructId) -> Vec<LawInfo<'a>> {
        let mut laws = self.structs[sid].laws.clone();
        let name = self.structs[sid].def.name.as_str();
        for imp in &self.impls {
            let (Some(tname), ast::TypeKind::Named { name: target, .. }) =
                (&imp.trait_name, &imp.target.kind)
            else {
                continue;
            };
            let Some(tr) = self.traits.iter().find(|t| &t.name == tname) else {
                continue;
            };
            if target.text != name {
                continue;
            }
            let analysis: &'a Analysis = self.mods[tr.mod_id];
            let stmts = analysis
                .items
                .iter()
                .filter(|item| item.offset == tr.offset)
                .flat_map(|item| item.syntax.parsed.stmts.iter());
            for stmt in stmts {
                let ast::StmtKind::Trait {
                    name: own, body, ..
                } = &stmt.kind
                else {
                    continue;
                };
                if &own.text != tname {
                    continue;
                }
                for law in &body.stmts {
                    if let ast::StmtKind::Law { name, params, body } = &law.kind {
                        laws.push(LawInfo {
                            mod_id: tr.mod_id,
                            offset: tr.offset,
                            name: name.text.clone(),
                            params,
                            body,
                        });
                    }
                }
            }
        }
        laws
    }

    /// `xs.m(op)` in a stress build, for a sequence `xs` and an `op` whose type has laws:
    /// `laws_hold(op, xs)` checked, then the call.
    pub(in crate::lower) fn checking_laws(
        &mut self,
        base: &ast::Expr,
        op: &ast::Arg,
        span: Span,
        call: impl FnOnce(&mut Self) -> Expr,
    ) -> Option<Expr> {
        if self.stress != Stress::Laws || op.name.is_some() {
            return None;
        }
        let Some(Ty::Struct(sid)) = self.probe_ty_silent(&op.value) else {
            return None;
        };
        self.probe_ty_silent(base).and_then(|t| t.seq_elem())?;
        if self.struct_laws(sid).is_empty() {
            return None;
        }
        self.stress = Stress::Off;
        let samples = ast::Arg {
            name: None,
            mutable: false,
            value: base.clone(),
        };
        let check = self.lower_laws_hold(&[op.clone(), samples], span);
        let folded = call(self);
        self.stress = Stress::Laws;
        let ty = folded.ty.clone();
        Some(self.mk(
            ExprKind::Do(Block {
                stmts: vec![Stmt::Expr(check), Stmt::Give(folded)],
            }),
            ty,
            span,
        ))
    }
}
