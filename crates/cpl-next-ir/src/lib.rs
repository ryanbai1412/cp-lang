//! The new pipeline's IR (contracts 4 and 5): the types every layer shares, the generic and
//! canonical IR, the linked program, and what each operation means. The checker, the
//! backends and the engines read it; it depends on nothing of theirs.
pub mod build;
pub mod facts;
pub mod ids;
pub mod ir;
pub mod layout;
pub mod msg;
pub mod ops;
pub mod opt;
pub mod prim;
pub mod prog;
pub mod run;
pub mod ty;
pub mod unbuilt;
pub mod verify;
pub mod whole;
