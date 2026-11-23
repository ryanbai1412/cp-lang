//! What the bounds prover shows in hovers, per revision of a file.

use std::sync::{Arc, OnceLock};

/// The bounds prover's results on one revision of a file, each computed on first use.
pub(super) struct Proven {
    pub(super) analysis: Arc<cpl_check::Analysis>,
    ranges: OnceLock<Vec<(u32, i128, i128)>>,
    sizes: OnceLock<Vec<cpl_lint::ProvenSize>>,
    domains: OnceLock<Vec<(String, u128)>>,
    fns: OnceLock<Vec<cpl_lint::ProvenFn>>,
    loops: OnceLock<Vec<cpl_lint::ProvenLoop>>,
    indexes: OnceLock<Vec<cpl_lint::ProvenIndex>>,
    exprs: OnceLock<Vec<cpl_lint::ProvenExpr>>,
}

impl Proven {
    pub(super) fn new(analysis: Arc<cpl_check::Analysis>) -> Proven {
        Proven {
            analysis,
            ranges: OnceLock::new(),
            sizes: OnceLock::new(),
            domains: OnceLock::new(),
            fns: OnceLock::new(),
            loops: OnceLock::new(),
            indexes: OnceLock::new(),
            exprs: OnceLock::new(),
        }
    }

    pub(super) fn text(&self) -> &str {
        &self.analysis.text
    }

    pub(super) fn with_items<T: Default>(
        &self,
        f: impl FnOnce(&str, &[(u32, &cpl_syntax::ParsedItem)]) -> T,
    ) -> T {
        if self.analysis.has_syntax_errors {
            return T::default();
        }
        let items: Vec<(u32, &cpl_syntax::ParsedItem)> = self
            .analysis
            .items
            .iter()
            .map(|i| (i.offset, &i.syntax.parsed))
            .collect();
        f(&self.analysis.text, &items)
    }

    pub(super) fn ranges(&self) -> &[(u32, i128, i128)] {
        self.ranges
            .get_or_init(|| self.with_items(cpl_lint::proven_ranges_parsed))
    }

    pub(super) fn sizes(&self) -> &[cpl_lint::ProvenSize] {
        self.sizes
            .get_or_init(|| self.with_items(cpl_lint::proven_sizes_parsed))
    }

    pub(super) fn domains(&self) -> &[(String, u128)] {
        self.domains
            .get_or_init(|| self.with_items(cpl_lint::domain_sizes_parsed))
    }

    pub(super) fn fns(&self) -> &[cpl_lint::ProvenFn] {
        self.fns
            .get_or_init(|| self.with_items(cpl_lint::proven_fns_parsed))
    }

    pub(super) fn loops(&self) -> &[cpl_lint::ProvenLoop] {
        self.loops_and_indexes();
        self.loops
            .get_or_init(|| self.with_items(cpl_lint::proven_loops_parsed))
    }

    pub(super) fn indexes(&self) -> &[cpl_lint::ProvenIndex] {
        self.loops_and_indexes();
        self.indexes
            .get_or_init(|| self.with_items(cpl_lint::proven_indexes_parsed))
    }

    /// A hover asks for the loops and then the indexes, and each proves the
    /// whole file, so the first ask proves both in one pass.
    fn loops_and_indexes(&self) {
        if self.loops.get().is_some() || self.indexes.get().is_some() {
            return;
        }
        let (loops, indexes) = self.with_items(cpl_lint::proven_loops_and_indexes_parsed);
        let _ = self.loops.set(loops);
        let _ = self.indexes.set(indexes);
    }

    pub(super) fn exprs(&self) -> &[cpl_lint::ProvenExpr] {
        self.exprs
            .get_or_init(|| self.with_items(cpl_lint::proven_expr_ranges_parsed))
    }
}
