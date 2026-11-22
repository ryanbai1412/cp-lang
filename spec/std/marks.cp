//! Opt-in generation-stamped mark table: `add`, `in`, `remove`, and `clear` are all O(1).
//!
//! For sparse resets prefer `[D]Bool` with a touched list; import `std.marks.Marks<D>` when
//! repeated full clears inside a loop would otherwise cost O(T * N). Each `clear()` bumps a
//! 32-bit generation counter instead of touching the underlying array, resetting the table
//! only if the counter reaches its 32-bit ceiling.

use std.interval.{IntervalMap}

const MAX_GEN = 2_000_000_000

/// A visited/mark set over a domain `D` with O(1) `add`, `remove`, `contains` (`v in m`),
/// and O(1) `clear()`. Memory is one `[D]0..=2_000_000_000` array (4 bytes per element).
pub struct Marks<D>:
    stamp: [D]0..=2_000_000_000
    gen: Int
    cnt: Int

    /// An empty mark table over `D`. O(D.K) once; subsequent `clear()` calls are O(1).
    pub fn new() -> Marks<D> = todo

    /// Mark `v` in the current round. Returns `true` iff `v` was not already marked. O(1).
    pub fn add(mut self, v: D) -> Bool = todo

    /// Unmark `v` in the current round. Returns `true` iff `v` was marked. O(1).
    pub fn remove(mut self, v: D) -> Bool = todo

    /// Is `v` marked in the current round? Also `v in m`. O(1).
    pub fn contains(self, v: D) -> Bool = todo

    /// Clear every mark in O(1) without touching the array (resets the table on wraparound).
    pub fn clear(mut self) = todo

    /// How many positions are marked in the current round. O(1).
    pub @get fn len(self) -> Int = todo

    /// Whether no positions are currently marked. O(1).
    pub @get fn is_empty(self) -> Bool = todo

/// Epoch-stamped map over domain `D` with O(1) `get`, `set`, `contains`, `remove`, and `clear()`.
pub struct MarkMap<D, T>:
    stamp: [D]0..=2_000_000_000
    vals: [D]T
    default_val: T
    gen: Int
    cnt: Int

    /// Create an empty `MarkMap<D, T>` backed by `default_val` for unset slots. O(D.K).
    pub fn new(default_val: T) -> MarkMap<D, T> = todo

    /// Number of active entries in the current round. O(1).
    pub @get fn len(self) -> Int = todo

    /// Whether no entries are active in the current round. O(1).
    pub @get fn is_empty(self) -> Bool = todo

    /// Whether `v` is present in the current round. O(1).
    pub fn contains(self, v: D) -> Bool = todo

    /// Current value at `v`, or `None` if absent in the current round. O(1).
    pub fn get(self, v: D) -> ?T = todo

    /// Set `v` to `val` in the current round. O(1).
    pub fn set(mut self, v: D, val: T) = todo

    /// Remove `v` from the current round; returns its previous value or `None`. O(1).
    pub fn remove(mut self, v: D) -> ?T = todo

    /// Clear every entry in O(1) without touching the value array.
    pub fn clear(mut self) = todo

/// Online value-to-id interner assigning contiguous `0..len` IDs to distinct keys `K`.
/// `id`, `get`, and `key` are O(1) expected; `len`, `is_empty`, `clear` are O(1).
pub struct Interner<K>:
    to_id: HashMap<K, Int>
    keys: List<K>

    /// Create an empty `Interner<K>`. O(1).
    pub fn new() -> Interner<K> = todo

    /// Number of distinct interned keys. O(1).
    pub @get fn len(self) -> Int = todo

    /// Whether no keys have been interned. O(1).
    pub @get fn is_empty(self) -> Bool = todo

    /// Remove all interned keys. O(1).
    pub fn clear(mut self) = todo

    /// Return the ID of `k`, assigning the next contiguous ID if `k` is new. O(1) expected.
    pub fn id(mut self, k: K) -> Int = todo

    /// Look up the ID of `k` without inserting, or `None` if unseen. O(1) expected.
    pub fn get(self, k: K) -> ?Int = todo

    /// Recover the key assigned to `i`. O(1).
    pub fn key(self, i: Int) -> K = todo

    /// Whether `k` has already been interned. O(1) expected.
    pub fn contains(self, k: K) -> Bool = todo

    /// Preallocate storage for at least `cap` distinct keys. O(cap).
    pub fn reserve(mut self, cap: Int) = todo

/// Briggs-Torczon sparse set over domain `D`.
/// O(1) `add`, `remove`, `contains`, `clear` AND O(`len`) iteration over active elements.
pub struct SparseSet<D>:
    _len: Int

    /// Create an empty `SparseSet<D>`. O(D.K).
    pub fn new() -> SparseSet<D> = todo

    /// Number of active elements. O(1).
    pub @get fn len(self) -> Int = todo

    /// Whether no elements are active. O(1).
    pub @get fn is_empty(self) -> Bool = todo

    /// Whether `v` is currently in the set. O(1).
    pub fn contains(self, v: D) -> Bool = todo

    /// Insert `v`; returns true if `v` was not already present. O(1).
    pub fn add(mut self, v: D) -> Bool = todo

    /// Remove `v`; returns true if `v` was present. O(1).
    pub fn remove(mut self, v: D) -> Bool = todo

    /// Clear all elements in O(1) without touching the sparse table.
    pub fn clear(mut self) = todo

    /// Yield all currently active elements in arbitrary order. O(`len`).
    pub fn each(self) -> yields D = todo

/// Compact 4-byte typed handle into an `Arena<T>`.
pub struct Id<T>(raw: 0..=2_000_000_000)

/// Contiguous pool allocator returning compact 4-byte `Id<T>` handles (for user trees, tries, and persistent nodes).
pub struct Arena<T>:
    _xs: List<T>

    /// Create an empty `Arena<T>` with optional initial capacity `cap`. O(cap).
    pub fn new(cap: Int = 0) -> Arena<T> = todo

    /// Number of allocated nodes. O(1).
    pub @get fn len(self) -> Int = todo

    /// Whether no nodes have been allocated. O(1).
    pub @get fn is_empty(self) -> Bool = todo

    /// Allocate `x` and return its compact handle. O(1) amortized.
    pub fn alloc(mut self, x: T) -> Id<T> = todo

    /// Read the node at `id` (W8 `Indexed` backing `arena[id]`). O(1).
    pub fn get(self, id: Id<T>) -> T = todo

    /// Overwrite the node at `id` (W8 `IndexSet` backing `arena[id] = x`). O(1).
    pub fn set(mut self, id: Id<T>, x: T) = todo

    /// Reserve backing storage for at least `cap` nodes. O(cap).
    pub fn reserve(mut self, cap: Int) = todo

    /// Reset the arena to empty (invalidates existing handles). O(1).
    pub fn clear(mut self) = todo

@test
fn mark_map_and_interner() = todo

@test
fn marks_basic_operations_and_clear() = todo

@test
fn marks_matches_bool_array_across_many_rounds() = todo

@test
fn marks_over_char_and_one_based_domains() = todo

@test
fn marks_wraps_cleanly_at_32bit_ceiling() = todo

@test
fn interval_map_cross_module_assign() = todo
