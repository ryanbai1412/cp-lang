//! Advice for habits from other languages, one module per language: each
//! names the CP Lang form of a call or test written as another language
//! writes it.

mod c;
mod python;
mod rust;

pub(crate) use c::*;
pub(crate) use python::*;
pub(crate) use rust::*;

use super::advice::*;
use super::*;
use cpl_diag::habits;
use cpl_diag::msgs;
use cpl_diag::Fix;
use cpl_syntax::ast::{CmpOp, Expr, Name};

/// Where the first argument of `rest`, the text just past a `(`, ends: at
/// the call's close (`true`) or at a comma (`false`).
pub(crate) fn first_arg(rest: &str) -> Option<(usize, bool)> {
    let mut depth = 0u32;
    for (i, c) in rest.char_indices() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' if depth == 0 => return Some((i, true)),
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => return Some((i, false)),
            _ => {}
        }
    }
    None
}
