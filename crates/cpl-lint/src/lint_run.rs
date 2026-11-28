//! One checked revision of a file and what lint works out from it, once:
//! the checker's facts, the bounds proof, and the findings. `cpl lint`, the
//! editor's diagnostics and hovers, and `cpl fix` read them from here, so the
//! file is proven once per revision however many of them ask.

use std::sync::{Arc, OnceLock};

use cpl_diag::Diagnostic;
use cpl_facade::today::Analysis;

use crate::{
    rules, Config, Facts, File, ProvenExpr, ProvenFn, ProvenIndex, ProvenLoop, ProvenSize,
};

pub struct LintRun {
    analysis: Arc<Analysis>,
    proven: OnceLock<rules::Proven>,
    derived: OnceLock<Derived>,
}

/// What lint works out from the proof that is not a part of it.
#[derive(Default)]
struct Derived {
    sizes: Vec<ProvenSize>,
    domains: Vec<(String, u128)>,
    fns: Vec<ProvenFn>,
}

impl LintRun {
    pub fn new(analysis: Arc<Analysis>) -> LintRun {
        LintRun {
            analysis,
            proven: OnceLock::new(),
            derived: OnceLock::new(),
        }
    }

    pub fn analysis(&self) -> &Arc<Analysis> {
        &self.analysis
    }

    pub fn text(&self) -> &str {
        &self.analysis.text
    }

    /// What the checker knows about each place in the file.
    pub fn facts(&self) -> &Facts {
        self.analysis.facts()
    }

    /// The lints at the levels `config` sets, as [`crate::lint_analysis`].
    pub fn lints(&self, config: &Config) -> Vec<Diagnostic> {
        crate::typed::lints_of(&self.analysis, self.facts(), &self.proven, config)
    }

    /// `f` on the file; a file with a syntax error gives `T::default()`.
    fn file<T: Default>(&self, f: impl FnOnce(&File) -> T) -> T {
        if self.analysis.has_syntax_errors {
            return T::default();
        }
        let items: Vec<_> = (self.analysis.items.iter())
            .map(|i| (i.offset, &i.syntax.parsed))
            .collect();
        f(&File {
            src: &self.analysis.text,
            items: &items,
            facts: Some(self.facts()),
            proven: &self.proven,
        })
    }

    /// The bounds proof; none for a file with a syntax error.
    fn proven(&self) -> Option<&rules::Proven> {
        if self.proven.get().is_none() {
            self.file(|f| {
                rules::proven(f);
            });
        }
        self.proven.get()
    }

    fn derived(&self) -> &Derived {
        self.derived.get_or_init(|| {
            self.file(|f| Derived {
                sizes: rules::proven_sizes(f),
                domains: rules::domain_sizes(f),
                fns: rules::proven_fns(f),
            })
        })
    }

    /// As [`crate::proven_ranges`].
    pub fn ranges(&self) -> &[(u32, i128, i128)] {
        self.proven().map_or(&[], |p| &p.names)
    }

    /// As [`crate::proven_sizes`].
    pub fn sizes(&self) -> &[ProvenSize] {
        &self.derived().sizes
    }

    /// As [`crate::domain_sizes`].
    pub fn domains(&self) -> &[(String, u128)] {
        &self.derived().domains
    }

    /// As [`crate::proven_fns`].
    pub fn fns(&self) -> &[ProvenFn] {
        &self.derived().fns
    }

    /// As [`crate::proven_loops`].
    pub fn loops(&self) -> &[ProvenLoop] {
        self.proven().map_or(&[], |p| &p.loops)
    }

    /// As [`crate::proven_indexes`].
    pub fn indexes(&self) -> &[ProvenIndex] {
        self.proven().map_or(&[], |p| &p.indexes)
    }

    /// As [`crate::proven_expr_ranges`].
    pub fn exprs(&self) -> &[ProvenExpr] {
        self.proven().map_or(&[], |p| &p.exprs)
    }
}
