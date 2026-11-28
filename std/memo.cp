//! The table a `@memo(word)` fn keeps its results in: a `@memo_table(word)` struct with an
//! ordinary impl of `MemoTable` (R06.4.1).
//!
//! start: `MemoTable`

/// The table a `@memo(word)` fn keeps (R06.4.1): `get` answers a key it has kept and `set`
/// keeps one. A `@memo_table(word)` struct is one through an ordinary impl; `f.clear()` builds
/// a fresh table, so there is no third member.
@role(memo_table)
pub trait MemoTable<K, V>:
    fn get(self, k: K) -> ?V
    fn set(mut self, k: K, v: V)
