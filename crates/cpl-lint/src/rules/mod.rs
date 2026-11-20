//! The rules. Each is a function from a parsed file to what it found, in
//! file positions; `lint` applies levels and `@allow` afterwards.

mod bounds;
mod fields;
mod floats;
mod lists;
mod memo;
mod scopes;
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
    findings
}
