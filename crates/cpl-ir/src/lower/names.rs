//! The names a lowered fn shows in traces, call stacks and the debugger,
//! and the prelude names a call keeps beside std's fold of that name.

use super::call::Reduce;
use super::Lowerer;
use crate::Expr;
use cpl_diag::Span;
use cpl_syntax::ast::{self, Arg, ExprKind};

/// `min(a, b)` and `count(x > 1 for x in xs)` call the core prelude fn even
/// where std.seq's fold of that name is in scope (R11.3.3, decision 53).
pub(super) fn core_over_std(name: &str, args: &[Arg]) -> bool {
    let positional = |a: &Arg| a.name.is_none();
    match name {
        "min" | "max" => args.len() >= 2 && args.iter().all(positional),
        "count" | "all" | "any" => matches!(args, [a]
            if positional(a) && matches!(a.value.kind, ExprKind::Comprehension { .. })),
        _ => false,
    }
}

impl Lowerer<'_> {
    /// `s.max()` or `s.min()`; `optional` gives `?T`, `None` when empty (under
    /// `??` or `!`, and always on a slice, which can be empty: R07.9.3).
    pub(super) fn min_max_reduce(
        &mut self,
        max: bool,
        seq: &ast::Expr,
        optional: bool,
        span: Span,
    ) -> Expr {
        if let Some(ends) = self.range_min_max(max, seq, optional, span) {
            return ends;
        }
        let kind = if max { Reduce::Max } else { Reduce::Min };
        self.reduce_opt(kind, seq, optional, span)
    }

    /// Per fn: whether a library module declares it, and its source name
    /// (`Heap.pop` for the method lowered as `Heap_Tup2_Int_P0_10.pop`).
    pub(super) fn fn_names(&self) -> (Vec<bool>, Vec<String>) {
        let sigs = self.sigs.iter().take(self.bodies.len());
        let library = sigs.clone().map(|sig| sig.mod_id != 0).collect();
        let shown = sigs
            .map(|sig| {
                let name = &sig.decl.name.text;
                if name.is_empty() || sig.name.starts_with("cpl_init_g") {
                    return sig.name.clone();
                }
                match sig.owner {
                    Some(id) => {
                        let s = &self.structs[id];
                        let owner = s.decl.map_or(s.def.name.as_str(), |d| d.name.text.as_str());
                        format!("{owner}.{name}")
                    }
                    None => name.clone(),
                }
            })
            .collect();
        (library, shown)
    }
}
