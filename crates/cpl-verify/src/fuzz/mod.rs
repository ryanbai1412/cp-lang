//! Random programs with a known answer.
//!
//! [`gen`] writes a program, [`eval`] is the reference evaluator that
//! says what it must print, [`print`] turns it into `.cp` source for a
//! backend, and [`shrink`] cuts a failing program down to a small one.

pub mod ast;
pub mod check;
pub mod eval;
pub mod gen;
pub mod print;
pub mod rng;
pub mod shrink;
