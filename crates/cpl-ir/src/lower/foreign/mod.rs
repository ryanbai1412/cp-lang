//! Recovery for habits from other languages in lowering, one module per
//! language: a call written as another language writes it gets the
//! CP Lang form as its fix.

mod c;
mod javascript;
mod python;

pub(in crate::lower) use c::foreign_insert;

use super::*;
use cpl_diag::msgs;
