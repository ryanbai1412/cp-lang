//! Diagnostics in two publishes: the checker's as soon as a document
//! changes, then, from [`Server::idle`], the full set `cpl check` and
//! `cpl lint` report, so the lints and proofs never delay the first.

use super::*;

impl Server {
    /// Every reply to `message` at once: [`Server::respond`], with the
    /// first publish replaced by the full one from [`Server::idle`].
    pub fn handle(&mut self, message: &Value) -> Vec<Value> {
        let mut out = self.respond(message);
        for full in self.idle() {
            match out
                .iter_mut()
                .find(|m| m.at(&["params", "uri"]) == full.at(&["params", "uri"]))
            {
                Some(first) => *first = full,
                None => out.push(full),
            }
        }
        out
    }

    /// The work deferred by the last message: the full diagnostics of the
    /// document it changed. A transport calls it once the replies are sent.
    pub fn idle(&mut self) -> Vec<Value> {
        let Some(uri) = self.owed.take().filter(|uri| self.docs.contains_key(uri)) else {
            return Vec::new();
        };
        match catch_unwind(AssertUnwindSafe(|| self.diagnostics(&uri))) {
            Ok(published) => vec![published],
            Err(panic) => {
                let params = obj([("textDocument", obj([("uri", uri.as_str().into())]))]);
                self.recover("lints", &params, Value::Null, panic_text(&*panic))
            }
        }
    }

    /// The checker's diagnostics for a document that just changed; the rest
    /// are owed to [`Server::idle`].
    pub(super) fn first_diagnostics(&mut self, uri: &str) -> Value {
        self.owed = Some(uri.to_string());
        let Some(analysis) = self.db.analyze(uri) else {
            return publish(uri, Vec::new());
        };
        let items = analysis
            .diagnostics
            .iter()
            .map(|d| diagnostic_json(d, uri, &analysis.text, &analysis.lines))
            .collect();
        publish(uri, items)
    }

    /// The `publishDiagnostics` notification for a document.
    pub(super) fn diagnostics(&mut self, uri: &str) -> Value {
        let Some(facts) = self.proven(uri) else {
            return publish(uri, Vec::new());
        };
        let analysis = self.db.analyze(uri).expect("proven implies an analysis");
        // The same answer as `cpl check`: the checker's diagnostics, then
        // what lowering rejects; then, as `cpl lint` reports them, the lints.
        let items = cpl_ir::reported(&analysis)
            .iter()
            .chain(&lints(uri, &analysis))
            .chain(&overflows(&facts))
            .map(|d| diagnostic_json(d, uri, &analysis.text, &analysis.lines))
            .collect();
        publish(uri, items)
    }
}

/// A soft note on each operation the prover shows leaves `Int` for inputs
/// in their declared ranges, naming those inputs.
fn overflows(facts: &Proven) -> Vec<Diagnostic> {
    let text = facts.text();
    facts
        .exprs()
        .iter()
        .filter_map(|e| {
            let inputs = e.overflow.as_ref()?;
            let span = Span::new(e.expr.0, e.expr.1);
            let source = text.get(e.expr.0 as usize..e.expr.1 as usize)?;
            let source = source.split_whitespace().collect::<Vec<_>>().join(" ");
            let mut diag = Diagnostic::warning(
                cpl_diag::codes::OVERFLOW,
                span,
                format!("`{source}` overflows Int when {inputs}"),
            );
            diag.severity = cpl_diag::Severity::Note;
            Some(diag)
        })
        .collect()
}

#[cfg(target_arch = "wasm32")]
pub(super) fn lints(_uri: &str, _analysis: &cpl_check::Analysis) -> Vec<Diagnostic> {
    Vec::new()
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn lints(uri: &str, analysis: &cpl_check::Analysis) -> Vec<Diagnostic> {
    cpl_lint::lint_analysis(analysis, &lint_config(uri))
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn lint_config(uri: &str) -> cpl_lint::Config {
    path_of_uri(uri)
        .map(|path| cpl_lint::Config::discover(std::path::Path::new(&path)).0)
        .unwrap_or_default()
}
