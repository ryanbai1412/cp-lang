//! What `cpl explain` reads off a write of the judge build's C++.

use std::collections::BTreeSet;

use super::Emitter;
use cpl_ir::Expr;

impl Emitter<'_> {
    /// One write's C++, with [`Emitter::copies`] and
    /// [`Emitter::called_and_written`] read off it.
    #[allow(clippy::type_complexity)]
    pub(crate) fn lowered(
        mut self,
    ) -> (
        String,
        Vec<(cpl_diag::Span, String, &'static str)>,
        BTreeSet<usize>,
        Written,
    ) {
        let cpp = self.write("", None);
        let copies = crate::lend::copy_sites(self.program, &self.copies);
        let direct = std::mem::take(&mut self.direct);
        (cpp, copies, direct, self.take_written())
    }

    /// Each index the written file has, and whether its C++ checks it.
    pub fn index_checks(mut self) -> Vec<(cpl_diag::Span, crate::explain::Index)> {
        self.write("", None);
        std::mem::take(&mut self.index_checks)
    }

    /// Records the index at `span`: `checked` when its C++ checks it, and
    /// otherwise why it needs no check. Library bodies' spans index their
    /// own files, so their indices are left out.
    pub(crate) fn note_index(
        &mut self,
        span: cpl_diag::Span,
        checked: bool,
        indices: &[Expr],
        raw: bool,
    ) {
        if self
            .fn_id
            .is_some_and(|f| self.program.library_fns.get(f).copied().unwrap_or(false))
        {
            return;
        }
        let index = match () {
            _ if checked => crate::explain::Index::Checked,
            _ if raw => crate::explain::Index::Raw,
            _ if crate::expr::proven(indices) => crate::explain::Index::Typed,
            _ => crate::explain::Index::Hoisted,
        };
        self.index_checks.push((span, index));
    }

    /// The closures the written file calls by name, and what it defines.
    pub fn called_and_written(mut self) -> (BTreeSet<usize>, Written) {
        self.write("", None);
        (std::mem::take(&mut self.direct), self.take_written())
    }

    fn take_written(&mut self) -> Written {
        std::mem::take(&mut self.written)
    }
}

/// The structs and fns a write of the C++ defines.
#[derive(Default)]
pub struct Written {
    pub structs: BTreeSet<usize>,
    pub fns: BTreeSet<usize>,
    pub closures: BTreeSet<usize>,
    /// The closures written as a function beside a copy of their captures
    /// (`cpl::bind_env`); every other local closure reads them in place.
    pub bound: BTreeSet<usize>,
}

impl Written {
    pub(super) fn of(
        structs: &BTreeSet<usize>,
        fns: &BTreeSet<usize>,
        closures: &BTreeSet<usize>,
    ) -> Written {
        Written {
            structs: structs.clone(),
            fns: fns.clone(),
            closures: closures.clone(),
            bound: BTreeSet::new(),
        }
    }
}
