//! `cpl doc --test`: the examples in doc comments must be programs.
//!
//! Each fenced block is parsed on its own. An example that no longer
//! parses is one the reader cannot copy, which is the drift this guards
//! against. Running them and comparing output waits on the interpreter;
//! [`Failure`] is what that will report too.

use crate::Module;
use cpl_diag::Diagnostic;

/// An example that is not a program.
#[derive(Clone, PartialEq, Debug)]
pub struct Failure {
    /// The item whose comment holds the example: `sum`, `Prefix.sum`, or
    /// the module's name for its own comment.
    pub item: String,
    /// The 1-based line of the file the example starts on.
    pub line: u32,
    pub code: String,
    /// Positions are within `code`.
    pub diagnostics: Vec<Diagnostic>,
}

/// How many examples were checked, and the ones that failed.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Report {
    pub checked: usize,
    pub failures: Vec<Failure>,
}

/// Checks every example of a module.
pub fn check(module: &Module) -> Report {
    let mut report = Report::default();
    let own = module.examples.iter().map(|e| (module.name.clone(), e));
    let items = module.all_items().flat_map(|(owner, item)| {
        let name = item.anchor(owner);
        item.examples.iter().map(move |e| (name.clone(), e))
    });
    for (item, example) in own.chain(items) {
        report.checked += 1;
        let diagnostics = cpl_syntax::parse_file(&example.code).diagnostics;
        if !diagnostics.is_empty() {
            report.failures.push(Failure {
                item,
                line: example.line,
                code: example.code.clone(),
                diagnostics,
            });
        }
    }
    report
}
