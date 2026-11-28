//! `Raw<T>`: a run of cells with no length or bounds, what `List`, `Str` and `Array` store
//! their values in. Each fn is one machine operation; the checked types build on them.
//!
//! start: `raw_new`, `raw_get`, `raw_set`, `raw_take`
//! example: `use std.raw.{raw_new, raw_get, raw_set}; var r = raw_new(3, 0); raw_set(mut r, 1, 5); raw_get(r, 1) == 5`

/// No cells.
pub fn raw_empty<T>() -> Raw<T> = @intrinsic("raw_empty")
/// `n` cells, each a copy of `fill`.
pub fn raw_new<T>(n: Int, fill: T) -> Raw<T> = @intrinsic("raw_new")
/// The value in cell `i`, unchecked: `i` is below the run's length and the cell is set.
pub fn raw_get<T>(r: Raw<T>, i: Int) -> T = @intrinsic("raw_get")
/// Puts `x` in cell `i`.
pub fn raw_set<T>(mut r: Raw<T>, i: Int, x: T) = @intrinsic("raw_set")
/// Moves cell `i` out of `r`, leaving it unset until `raw_set` fills it.
pub fn raw_take<T>(mut r: Raw<T>, i: Int) -> T = @intrinsic("move_out")
/// Storage of `n` unset cells; each is filled by `raw_set` before it is read.
pub fn raw_cap<T>(n: Int) -> Raw<T> = @intrinsic("unset_cells")
/// Traps R0101 unless `ok`: a read at a key the sequence doesn't have (R07.2.17).
pub fn check_key(ok: Bool) = @intrinsic("key_check")
/// Traps unless `ok`: a range that is not inside the sequence it slices.
pub fn check_range(ok: Bool) = @intrinsic("range_check")
