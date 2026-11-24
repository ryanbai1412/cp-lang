//! What `cpl explain` reads off a write of the judge build's C++.

use std::collections::BTreeSet;

use super::Emitter;

impl Emitter<'_> {
    /// The copies [`Emitter::file`] writes, each with where and why.
    pub fn copies(mut self) -> Vec<(cpl_diag::Span, String, &'static str)> {
        self.write("", None);
        crate::lend::copy_sites(self.program, &self.copies)
    }

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
}

impl Written {
    pub(super) fn of(structs: &BTreeSet<usize>, fns: &BTreeSet<usize>) -> Written {
        Written {
            structs: structs.clone(),
            fns: fns.clone(),
        }
    }
}
