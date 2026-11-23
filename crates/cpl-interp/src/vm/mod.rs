//! Typed register bytecode VM for plain (untraced, unprofiled) runs.
//!
//! `cpl-ir` already resolves every local to a `Slot` and gives every slot,
//! global, and expression a static [`Ty`]. The VM exploits that static typing:
//! - Scalar values (`Int`, `Pos`, `Bound`, `Bool`, `Char`, and 1-field `Int`
//!   structs like `Mod<P>`) live unboxed in a flat `Vec<i64>` register bank
//!   (`regs_i`) with zero `Value` tagging, cloning, or dropping.
//! - Optional scalars (`?Int`, `?Pos`, `?Bound`, `?Bool`, `?Char`, `?Mod<P>`)
//!   live unboxed as a `(val, is_some)` pair of `i64` registers, backed by
//!   `ArrayData::OptInt` for `@memo` and DP tables.
//! - Compound values (`Array`, `List`, `Str`, `Map`, `Set`, multi-field
//!   tuples/structs/enums, closures) live in `regs_v: Vec<Value>`.
//! - Small pure leaf functions (e.g. `Mod<P>` arithmetic) are inlined into the
//!   caller's bytecode stream at compile time.
//! - Hot contest loops use superinstructions (`ForInc`, `ForIncClosed`,
//!   `ForDec`, `LoadGArr1I`/`StoreGArr1I`, `LoadGArr2I`/`StoreGArr2I`,
//!   `LoadGArr2OptI`/`StoreGArr2OptI`, `PosNext`/`PosPrev`/`PosOffset`,
//!   `JmpLtI`/`JmpLeI`/`JmpEqI`/`JmpNeI`).
//! - Any IR construct not lowered natively executes via `FallbackStmt` /
//!   `FallbackExpr`, syncing live local slots with `Interp` so 100% of CP Lang
//!   features work identically.

mod compile;
mod exec;
mod helpers;
mod ir;

pub(crate) use helpers::VmStats;
pub(crate) use helpers::{read_slot, undeclared};
pub(crate) use ir::CompiledProgram;
#[cfg(test)]
pub(crate) use ir::Op;
pub(crate) use ir::{Chunk, LoopReg, SeqHeld, SlotLoc};
