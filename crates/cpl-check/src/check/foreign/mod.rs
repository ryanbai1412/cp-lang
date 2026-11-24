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
