//! One checked revision of a file and what lint works out from it, once:
//! the checker's facts, the bounds proof, and the findings. `cpl lint`, the
//! editor's diagnostics and hovers, and `cpl fix` read them from here, so the
//! file is proven once per revision however many of them ask.

use std::sync::{Arc, OnceLock};

use cpl_check::Analysis;
use cpl_diag::Diagnostic;

use crate::{
    rules, Config, Facts, File, ProvenExpr, ProvenFn, ProvenIndex, ProvenLoop, ProvenSize,
};

pub struct Session {
    analysis: Arc<Analysis>,
    proven: OnceLock<rules::Proven>,
    ranges: OnceLock<Vec<(u32, i128, i128)>>,
    sizes: OnceLock<Vec<ProvenSize>>,
    domains: OnceLock<Vec<(String, u128)>>,
    fns: OnceLock<Vec<ProvenFn>>,
    loops: OnceLock<Vec<ProvenLoop>>,
    indexes: OnceLock<Vec<ProvenIndex>>,
    exprs: OnceLock<Vec<ProvenExpr>>,
}

impl Session {
    pub fn new(analysis: Arc<Analysis>) -> Session {
        Session {
            analysis,
            proven: OnceLock::new(),
            ranges: OnceLock::new(),
            sizes: OnceLock::new(),
            domains: OnceLock::new(),
            fns: OnceLock::new(),
            loops: OnceLock::new(),
            indexes: OnceLock::new(),
            exprs: OnceLock::new(),
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

    /// As [`crate::proven_ranges`].
    pub fn ranges(&self) -> &[(u32, i128, i128)] {
        self.ranges.get_or_init(|| self.file(rules::proven_ranges))
    }

    /// As [`crate::proven_sizes`].
    pub fn sizes(&self) -> &[ProvenSize] {
        self.sizes.get_or_init(|| self.file(rules::proven_sizes))
    }

    /// As [`crate::domain_sizes`].
    pub fn domains(&self) -> &[(String, u128)] {
        self.domains.get_or_init(|| self.file(rules::domain_sizes))
    }

    /// As [`crate::proven_fns`].
    pub fn fns(&self) -> &[ProvenFn] {
        self.fns.get_or_init(|| self.file(rules::proven_fns))
    }

    /// As [`crate::proven_loops`].
    pub fn loops(&self) -> &[ProvenLoop] {
        self.loops.get_or_init(|| self.file(rules::proven_loops))
    }

    /// As [`crate::proven_indexes`].
    pub fn indexes(&self) -> &[ProvenIndex] {
        self.indexes
            .get_or_init(|| self.file(rules::proven_indexes))
    }

    /// As [`crate::proven_expr_ranges`].
    pub fn exprs(&self) -> &[ProvenExpr] {
        self.exprs
            .get_or_init(|| self.file(rules::proven_expr_ranges))
    }
}
