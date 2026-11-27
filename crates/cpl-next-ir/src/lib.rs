//! The new pipeline's IR (contracts 4 and 5): the types every layer shares, the generic and
//! canonical IR, the linked program, and what each operation means. The checker, the
//! backends and the engines read it; it depends on nothing of theirs.
pub mod facts;
pub mod ir;
pub mod msg;
pub mod ops;
pub mod opt;
pub mod prog;
pub mod run;
pub mod ty;
