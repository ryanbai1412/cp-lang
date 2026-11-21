//! The rules. Each is a function from a parsed file to what it found, in
//! file positions; `lint` applies levels and `@allow` afterwards.

mod bounds;
mod fields;
mod floats;
mod lists;
mod memo;
mod scopes;
mod taste;
mod types;

use crate::{Config, File, Finding};

pub(crate) fn run(file: &File, config: &Config) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(scopes::check(file));
    findings.extend(types::check(file));
    findings.extend(floats::check(file));
    findings.extend(fields::check(file));
    findings.extend(memo::check(file, config.memory_mb));
    findings.extend(lists::check(file));
    findings.extend(taste::check(file));
    findings
}

/// Each top-level `let` without a written type whose value the declared
/// bounds prove, as the name's offset and the inclusive range.
pub(crate) fn proven_ranges(file: &File) -> Vec<(u32, i128, i128)> {
    use cpl_syntax::ast::{PatternKind, StmtKind};
    let bounds = bounds::Bounds::of(file);
    let mut out = Vec::new();
    for (base, stmt) in file.top_level() {
        if let StmtKind::Let {
            mutable: false,
            pat,
            ty: None,
            ..
        } = &stmt.kind
        {
            if let PatternKind::Name(name) = &pat.kind {
                if let Some(value) = bounds.value(name) {
                    out.push((base + pat.span.start, value.lo, value.hi));
                }
            }
        }
    }
    out
}
