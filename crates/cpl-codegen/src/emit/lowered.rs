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
        BTreeSet<usize>,
    ) {
        let cpp = self.write("", None);
        let copies = crate::lend::copy_sites(self.program, &self.copies);
        let direct = std::mem::take(&mut self.direct);
        (
            cpp,
            copies,
            direct,
            std::mem::take(&mut self.written_structs),
        )
    }

    /// The closures the written file calls by name, and the structs it defines.
    pub fn called_and_written(mut self) -> (BTreeSet<usize>, BTreeSet<usize>) {
        self.write("", None);
        (
            std::mem::take(&mut self.direct),
            std::mem::take(&mut self.written_structs),
        )
    }
}
