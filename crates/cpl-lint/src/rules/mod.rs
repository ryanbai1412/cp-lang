//! The rules. Each is a function from a parsed file to what it found, in
//! file positions; `lint` applies levels and `with lint` afterwards.

mod answers;
mod assign;
mod bounds;
mod ceil;
mod euclid;
mod fields;
mod floats;
mod grid;
mod imports;
mod isqrt;
mod lists;
mod literals;
mod memo;
mod mod_div;
mod mod_reduce;
mod pops;
mod proven;
mod reverse;
mod scopes;
mod sentinel;
mod taste;
mod types;

use crate::{Config, File, Finding};

pub(crate) use proven::*;

pub(crate) fn run(file: &File, config: &Config) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(scopes::check(file));
    findings.extend(types::check(file));
    findings.extend(assign::check(file));
    findings.extend(answers::check(file));
    findings.extend(pops::check(file));
    findings.extend(ceil::check(file));
    findings.extend(euclid::check(file));
    findings.extend(reverse::check(file));
    findings.extend(isqrt::check(file));
    findings.extend(sentinel::check(file));
    findings.extend(mod_reduce::check(file));
    findings.extend(grid::check(file));
    findings.extend(floats::check(file));
    findings.extend(imports::check(file));
    findings.extend(fields::check(file));
    findings.extend(memo::check(file, config.memory_mb));
    findings.extend(lists::check(file));
    findings.extend(mod_div::check(file));
    findings.extend(taste::check(file));
    findings.extend(literals::check(file));
    findings
}
