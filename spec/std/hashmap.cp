//! Open-addressing hash map for `Int` keys with splitmix64 salting.
//!
//! `FlatMap<V>` stores keys, values, and occupancy states in flat parallel arrays with
//! linear probing and power-of-two capacity, avoiding per-node heap allocations when
//! built-in `HashMap<Int, V>` overhead matters.
use std.math.{splitmix64}

// Slot states: 0 = empty, 1 = occupied, 2 = tombstone.
/// Flat open-addressing hash map from `Int` keys to `V`, salted with `splitmix64`.
/// `get`, `contains`, `set`, `remove`, and `get_or_insert` are O(1) expected.
pub struct FlatMap<V>:
    keys: List<Int>
    vals: List<V>
    state: List<Int>
    default_val: V
    salt: U64
    live: Int
    used: Int

    /// [rename] Create an empty `FlatMap<V>` sized for at least `cap` entries (replaces `with_capacity`). O(cap).
    pub fn new(default_val: V, cap: Int = 16) -> FlatMap<V> = todo

    /// Number of live key-value pairs. O(1).
    pub @get fn len(self) -> Int = todo

    /// Whether the map has no live entries. O(1).
    pub @get fn is_empty(self) -> Bool = todo

    /// Remove all entries while keeping allocated capacity. O(capacity).
    pub fn clear(mut self) = todo

    fn slot(self, k: Int) -> Int = todo

    fn rehash(mut self, new_cap: Int) = todo

    /// Look up `k`, returning `none` if absent. O(1) expected.
    pub fn get(self, k: Int) -> ?V = todo

    /// Whether `k` is currently present. O(1) expected.
    pub fn contains(self, k: Int) -> Bool = todo

    /// Insert or overwrite `k -> v`. O(1) expected amortized.
    pub fn set(mut self, k: Int, v: V) = todo

    /// Return the value for `k`, inserting `v` first if `k` is absent. O(1) expected amortized.
    pub fn get_or_insert(mut self, k: Int, v: V) -> V = todo

    /// Remove `k` if present; returns its old value or `none`. O(1) expected.
    pub fn remove(mut self, k: Int) -> ?V = todo

    /// Reserve space for at least `cap` entries without rehashing. O(cap).
    pub fn reserve(mut self, cap: Int) = todo

    /// Move the map's contents out in O(1), leaving `self` empty.
    pub fn take(mut self) -> FlatMap<V> = todo

    /// Yield every live `(key, val)` pair. O(capacity).
    pub fn each(self) -> yields (Int, V) = todo

    /// Yield every live key. O(capacity).
    pub fn keys(self) -> yields Int = todo

    /// Yield every live value. O(capacity).
    pub fn values(self) -> yields V = todo

    /// Merge all entries from `other` into `self` (emptying `other`), combining collisions with `join` or overwriting. O(min(|self|, |other|)) expected.
    pub fn absorb(mut self, mut other: FlatMap<V>, join: ?((V, V) -> V) = none) = todo

/// Default-valued frequency / weight map over hashable keys `K`.
/// `c[k]` reads `init` without inserting; `add`, `get`, `set`, `remove`, and `contains` are O(1) expected.
pub struct Counter<K, V>:
    _len: Int

    /// Create an empty `Counter<K, V>` whose missing keys read as `init`. O(1).
    pub fn new(init: V) -> Counter<K, V> = todo

    /// Count occurrences of each element in `xs`. O(n) expected.
    pub fn of(xs: List<K>) -> Counter<K, Int> = todo

    /// Number of keys with a stored entry. O(1).
    pub @get fn len(self) -> Int = todo

    /// Whether no keys are stored. O(1).
    pub @get fn is_empty(self) -> Bool = todo

    /// Value stored for `k`, or `init` if absent (does not insert). O(1) expected.
    pub fn get(self, k: K) -> V = todo

    /// Whether `k` currently has a stored entry. O(1) expected.
    pub fn contains(self, k: K) -> Bool = todo

    /// Set `k` to `v` (W8 `IndexSet` backing `c[k] = v`). O(1) expected amortized.
    pub fn set(mut self, k: K, v: V) = todo

    /// Add `delta` to `k`'s value, removing `k` if the result equals `init`. O(1) expected amortized.
    pub fn add(mut self, k: K, delta: V) = todo

    /// Remove `k` and return its previous value, or `none` if absent. O(1) expected.
    pub fn remove(mut self, k: K) -> ?V = todo

    /// Reserve space for at least `cap` keys. O(cap).
    pub fn reserve(mut self, cap: Int) = todo

    /// Remove all entries. O(n).
    pub fn clear(mut self) = todo

    /// Move contents out in O(1), leaving `self` empty.
    pub fn take(mut self) -> Counter<K, V> = todo

    /// Merge all entries from `other` into `self` by adding values (emptying `other`). O(min(|self|, |other|)) expected.
    pub fn absorb(mut self, mut other: Counter<K, V>) = todo

    /// Sum of all stored values. O(n).
    pub fn total(self) -> V = todo

    /// Top `k` entries sorted by descending value. O(n log n).
    pub fn most_common(self, k: Int) -> List<(K, V)> = todo

    /// Yield every stored `(key, val)` pair. O(n).
    pub fn each(self) -> yields (K, V) = todo

    /// Yield every stored key. O(n).
    pub fn keys(self) -> yields K = todo

    /// Yield every stored value. O(n).
    pub fn values(self) -> yields V = todo

@test
fn flat_map_matches_builtin_hash_map() = todo

@test
fn hash_map_and_set_erase_clear_and_equality() = todo
