//! Ordered multiset and set with order statistics, range views, and cursors: `SortedBag<K>`, `SortedSet<K>`.

use std.seq.{remove}

/// Read-only handle into a `SortedBag<K>` node, valid until the next mutation.
pub struct BagCursor(id: Int)

struct BagNode<K>:
    key: K
    cnt: Int
    sub: Int
    pri: -2_147_483_648..=2_147_483_647
    left: -2_147_483_648..=2_147_483_647
    right: -2_147_483_648..=2_147_483_647
    par: -2_147_483_648..=2_147_483_647

/// An ordered multiset (bag) of `K` backed by a randomized treap over a single flat node array
/// with recycled node slots and parent pointers for O(1) amortized cursor stepping.
/// Keeps one node per distinct key with its multiplicity and subtree element count.
/// `add`, `add_many`, `remove`, `remove_all`, `contains`, `count`, `first`, `last`,
/// `pop_first`, `pop_last`, `rank`, `nth`, `first_true`, `last_true`, and `cursor_*` are O(log n) expected;
/// `range(r)` returns an O(1) view whose `.first`, `.last`, `.len`, `.is_empty` are O(log n)
/// expected and whose iteration (`for x in b.range(r)` / `.rev`) is O(log n + k);
/// `key_of` and `count_of` are O(1); `next_cursor` and `prev_cursor` are O(1) amortized;
/// `len`, `distinct`, `is_empty`, `clear`, and `take` are O(1);
/// `entries`, `entries_rev`, and `to_list` are O(n).
pub struct SortedBag<K>:
    nodes: List<BagNode<K>>
    free: List<-2_147_483_648..=2_147_483_647>
    live: Int
    root: Int
    rng: U64

    /// Create an empty `SortedBag<K>`. O(1).
    pub fn new() -> SortedBag<K> = todo

    /// Build a `SortedBag` containing every element of `xs`. O(n log n) expected.
    pub fn of(xs: List<K>) -> SortedBag<K> = todo

    /// Total number of elements in the bag (counting duplicates). O(1).
    pub @get fn len(self) -> Int = todo

    /// Number of distinct keys currently in the bag. O(1).
    pub @get fn distinct(self) -> Int = todo

    /// Whether the bag has no elements. O(1).
    pub @get fn is_empty(self) -> Bool = todo

    /// Remove all elements from the bag. O(1).
    pub fn clear(mut self) = todo

    /// Move the bag's contents out in O(1), leaving `self` empty.
    pub fn take(mut self) -> SortedBag<K> = todo

    /// Insert one copy of `x` into the bag. Always returns `true`. O(log n) expected.
    pub fn add(mut self, x: K) -> Bool = todo

    fn add_unique(mut self, x: K) -> Bool = todo

    /// Insert `c >= 0` copies of `x` into the bag. O(log n) expected.
    pub fn add_many(mut self, x: K, c: Int) = todo

    /// Remove one copy of `x` if present; returns `true` if a copy was removed. O(log n) expected.
    pub fn remove(mut self, x: K) -> Bool = todo

    /// Remove all copies of `x` and return how many were removed. O(log n) expected.
    pub fn remove_all(mut self, x: K) -> Int = todo

    /// How many copies of `x` are currently in the bag. O(log n) expected.
    pub fn count(self, x: K) -> Int = todo

    /// Whether at least one copy of `x` is in the bag. O(log n) expected.
    pub fn contains(self, x: K) -> Bool = todo

    /// The smallest element, or `none` when empty. O(1), kept current by every update.
    pub @get fn first(self) -> ?K = todo

    /// The largest element, or `none` when empty. O(1), kept current by every update.
    pub @get fn last(self) -> ?K = todo

    /// Remove and return one copy of the smallest element, or `none` when empty. O(log n) expected.
    pub fn pop_first(mut self) -> ?K = todo

    /// Remove and return one copy of the largest element, or `none` when empty. O(log n) expected.
    pub fn pop_last(mut self) -> ?K = todo

    /// Number of elements strictly less than `k` (counting duplicates). O(log n) expected.
    pub fn rank(self, k: K) -> Int = todo

    fn rank_le(self, k: K) -> Int = todo

    fn lo_ok(self, x: K, lo: ?K, lo_inc: Bool) -> Bool = todo

    fn hi_ok(self, x: K, hi: ?K, hi_inc: Bool) -> Bool = todo

    fn range_first(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> ?K = todo

    fn range_last(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> ?K = todo

    fn range_len(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> Int = todo

    fn range_each(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> yields K = todo

    fn range_each_rev(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> yields K = todo

    /// 0-indexed `i`-th smallest element (counting duplicates), or `none` if out of bounds. O(log n) expected.
    pub fn nth(self, i: Int) -> ?K = todo

    /// Smallest key where monotone `ok(k)` holds (`false..false, true..true`), or `none`.
    /// Checks monotonicity at the boundary. O(log n) expected.
    pub fn first_true(self, ok: K -> Bool) -> ?K = todo

    /// Largest key where monotone `ok(k)` holds (`true..true, false..false`), or `none`.
    /// Checks monotonicity at the boundary. O(log n) expected.
    pub fn last_true(self, ok: K -> Bool) -> ?K = todo

    /// Remove and return the maximal run of elements strictly after `after` (ascending),
    /// strictly before `before` (descending), or from the smallest element (ascending)
    /// while `ok(k)` holds. O(k log n) expected for `k` removed elements.
    pub fn drain(mut self; after: ?K = none, before: ?K = none, while ok: K -> Bool) -> List<K> = todo

    /// Cursor to the smallest key, or `none` when empty. O(log n) expected.
    pub fn cursor_first(self) -> ?BagCursor = todo

    /// Cursor to the largest key, or `none` when empty. O(log n) expected.
    pub fn cursor_last(self) -> ?BagCursor = todo

    /// Cursor to exact key `k`, or `none` if absent. O(log n) expected.
    pub fn cursor_at(self, k: K) -> ?BagCursor = todo

    /// Cursor to smallest key `>= k`, or `none`. O(log n) expected.
    pub fn cursor_ge(self, k: K) -> ?BagCursor = todo

    /// Cursor to smallest key `> k`, or `none`. O(log n) expected.
    pub fn cursor_gt(self, k: K) -> ?BagCursor = todo

    /// Cursor to largest key `<= k`, or `none`. O(log n) expected.
    pub fn cursor_le(self, k: K) -> ?BagCursor = todo

    /// Cursor to largest key `< k`, or `none`. O(log n) expected.
    pub fn cursor_lt(self, k: K) -> ?BagCursor = todo

    /// Key stored at cursor `c`. O(1).
    pub fn key_of(self, c: BagCursor) -> K = todo

    /// Multiplicity stored at cursor `c`. O(1).
    pub fn count_of(self, c: BagCursor) -> Int = todo

    /// Cursor to the next larger distinct key, or `none`. O(1) amortized.
    pub fn next_cursor(self, c: BagCursor) -> ?BagCursor = todo

    /// Cursor to the next smaller distinct key, or `none`. O(1) amortized.
    pub fn prev_cursor(self, c: BagCursor) -> ?BagCursor = todo

    /// Visit every distinct `(key, count)` in ascending order. O(1) amortized per step.
    pub fn entries(self) -> yields (key: K, count: Int) = todo

    /// Visit every distinct `(key, count)` in descending order. O(1) amortized per step.
    pub fn entries_rev(self) -> yields (key: K, count: Int) = todo

    /// Every element in ascending order, repeated by multiplicity. O(n).
    pub fn to_list(self) -> List<K> = todo

    /// Preallocate treap node pool for at least `cap` distinct keys. O(cap).
    pub fn reserve(mut self, cap: Int) = todo

    /// Split off and return all elements `>= k`, keeping `< k` in `self`. O(log n).
    pub fn split_at(mut self, k: K) -> SortedBag<K> = todo

    /// Concatenate `other` onto `self` when every key in `self` is `<` every key in `other`. O(log n).
    pub fn join(mut self, other: SortedBag<K>) = todo

    /// Merge all elements of `other` into `self` (small-to-large, emptying `other`). O(min(n, m) log(n + m)).
    pub fn absorb(mut self, mut other: SortedBag<K>) = todo

    /// Sum of all elements (for additive `K`). O(1) when augmented, O(n) otherwise.
    pub fn sum(self) -> Int = todo

    fn sub_of(self, t: Int) -> Int = todo

    fn pull(mut self, u: Int) = todo

    fn make_node(mut self, x: K, c: Int) -> Int = todo

    fn insert_at(mut self, u: Int, x: K, c: Int, unique: Bool) -> (Int, Bool) = todo

    fn erase_at(mut self, u: Int, x: K, all: Bool) -> (Int, Int) = todo

    fn merge(mut self, a: Int, b: Int) -> Int = todo

/// An ordered set of unique keys `K` with order statistics (`rank`, `nth`),
/// O(1) key-range views (`range(r)`), and monotone search (`first_true`/`last_true`). Backed by `SortedBag<K>`.
/// All single-key and range-summary operations are O(log n) expected;
/// `range(r)` is O(1); range iteration is O(log n + k);
/// `len`, `is_empty`, `clear`, `take` are O(1); `each`, `each_rev`, and `to_list` are O(n).
pub struct SortedSet<K>:
    bag: SortedBag<K>

    /// Create an empty `SortedSet<K>`. O(1).
    pub fn new() -> SortedSet<K> = todo

    /// Build a `SortedSet` containing the distinct elements of `xs`. O(n log n) expected.
    pub fn of(xs: List<K>) -> SortedSet<K> = todo

    /// Number of keys in the set. O(1).
    pub @get fn len(self) -> Int = todo

    /// Whether the set is empty. O(1).
    pub @get fn is_empty(self) -> Bool = todo

    /// Remove all keys from the set. O(1).
    pub fn clear(mut self) = todo

    /// Move all keys out, leaving `self` empty. O(1).
    pub fn take(mut self) -> SortedSet<K> = todo

    /// Insert `x` if absent; returns `true` when `x` was newly inserted. O(log n) expected.
    pub fn add(mut self, x: K) -> Bool = todo

    /// Remove `x` if present; returns `true` if removed. O(log n) expected.
    pub fn remove(mut self, x: K) -> Bool = todo

    /// Add `x` when `on` is true, or remove it otherwise, matching `Set.set` and `Bitset.set`. O(log n) expected.
    pub fn set(mut self, x: K, on: Bool) = todo

    /// Whether `x` is in the set. O(log n) expected.
    pub fn contains(self, x: K) -> Bool = todo

    /// Smallest key in the set, or `none`. O(1), kept current by every update.
    pub @get fn first(self) -> ?K = todo

    /// Largest key in the set, or `none`. O(1), kept current by every update.
    pub @get fn last(self) -> ?K = todo

    /// Remove and return the smallest key, or `none`. O(log n) expected.
    pub fn pop_first(mut self) -> ?K = todo

    /// Remove and return the largest key, or `none`. O(log n) expected.
    pub fn pop_last(mut self) -> ?K = todo

    /// Number of keys strictly less than `k`. O(log n) expected.
    pub fn rank(self, k: K) -> Int = todo

    /// 0-indexed `i`-th smallest key, or `none` if out of range. O(log n) expected.
    pub fn nth(self, i: Int) -> ?K = todo

    /// The sum of the elements in `keys` (all of them by default), for additive `K`:
    /// `set.sum(..b)` adds everything below `b`. O(log n): subtree sums are kept.
    pub fn sum(self, keys: Range<K> = ..) -> K = todo

    /// Smallest key `k` where monotone `ok(k)` holds (`false..false, true..true`), or `none`. O(log n) expected.
    pub fn first_true(self, ok: K -> Bool) -> ?K = todo

    /// Largest key `k` where monotone `ok(k)` holds (`true..true, false..false`), or `none`. O(log n) expected.
    pub fn last_true(self, ok: K -> Bool) -> ?K = todo

    /// Remove and return the maximal run of keys strictly after `after` (ascending),
    /// strictly before `before` (descending), or from the smallest key (ascending)
    /// while `ok(k)` holds. O(k log n) expected for `k` removed keys.
    pub fn drain(mut self; after: ?K = none, before: ?K = none, while ok: K -> Bool) -> List<K> = todo

    fn range_first(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> ?K = todo

    fn range_last(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> ?K = todo

    fn range_len(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> Int = todo

    fn range_each(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> yields K = todo

    fn range_each_rev(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> yields K = todo

    /// Visit every key in ascending order. O(1) amortized per step.
    pub fn each(self) -> yields K = todo

    /// Visit every key in descending order. O(1) amortized per step.
    pub fn each_rev(self) -> yields K = todo

    /// Return all keys in ascending order as a `List<K>`. O(n).
    pub fn to_list(self) -> List<K> = todo

    /// Preallocate treap node pool for at least `cap` keys. O(cap).
    pub fn reserve(mut self, cap: Int) = todo

    /// Split off and return all keys `>= k`, keeping `< k` in `self`. O(log n).
    pub fn split_at(mut self, k: K) -> SortedSet<K> = todo

    /// Concatenate `other` onto `self` when every key in `self` is `<` every key in `other`. O(log n).
    pub fn join(mut self, other: SortedSet<K>) = todo

    /// Merge all keys of `other` into `self` (small-to-large, emptying `other`). O(min(n, m) log(n + m)).
    pub fn absorb(mut self, mut other: SortedSet<K>) = todo

/// Order-statistic ordered map from `K` to `V` backed by a randomized treap.
/// `get`, `set`, `remove`, `rank`, `nth`, `first_ge`/`last_le`, `first_true`/`last_true`, `split_at`, `join` are O(log n).
pub struct SortedMap<K, V>:
    _len: Int

    /// Create an empty `SortedMap<K, V>`. O(1).
    pub fn new() -> SortedMap<K, V> = todo

    /// Preallocate treap node pool for at least `cap` entries. O(cap).
    pub fn reserve(mut self, cap: Int) = todo

    /// Number of key-value pairs. O(1).
    pub @get fn len(self) -> Int = todo

    /// Whether the map is empty. O(1).
    pub @get fn is_empty(self) -> Bool = todo

    /// Remove all entries. O(1).
    pub fn clear(mut self) = todo

    /// Move contents out in O(1), leaving `self` empty.
    pub fn take(mut self) -> SortedMap<K, V> = todo

    /// Look up `k`, returning `none` if absent. O(log n).
    pub fn get(self, k: K) -> ?V = todo

    /// Whether `k` is present. O(log n).
    pub fn contains(self, k: K) -> Bool = todo

    /// Insert or overwrite `k -> v` (W8 `IndexSet` backing `m[k] = v`). O(log n).
    pub fn set(mut self, k: K, v: V) = todo

    /// Remove `k` and return its previous value, or `none` if absent. O(log n).
    pub fn remove(mut self, k: K) -> ?V = todo

    /// Smallest `(key, val)` pair, or `none` if empty. O(1), kept current by every update.
    pub @get fn first(self) -> ?(K, V) = todo

    /// Largest `(key, val)` pair, or `none` if empty. O(1), kept current by every update.
    pub @get fn last(self) -> ?(K, V) = todo

    /// Remove and return the smallest `(key, val)` pair, or `none`. O(log n).
    pub fn pop_first(mut self) -> ?(K, V) = todo

    /// Remove and return the largest `(key, val)` pair, or `none`. O(log n).
    pub fn pop_last(mut self) -> ?(K, V) = todo

    /// Smallest entry with key `>= k`, or `none`. O(log n).
    pub fn first_ge(self, k: K) -> ?(K, V) = todo

    /// Smallest entry with key `> k`, or `none`. O(log n).
    pub fn first_gt(self, k: K) -> ?(K, V) = todo

    /// Largest entry with key `<= k`, or `none`. O(log n).
    pub fn last_le(self, k: K) -> ?(K, V) = todo

    /// Largest entry with key `< k`, or `none`. O(log n).
    pub fn last_lt(self, k: K) -> ?(K, V) = todo

    /// Number of entries with key strictly less than `k`. O(log n).
    pub fn rank(self, k: K) -> Int = todo

    /// The `i`-th entry in key order (0-based), or `none`. O(log n).
    pub fn nth(self, i: Int) -> ?(K, V) = todo

    /// Smallest entry where monotone `ok(k, v)` becomes true, or `none`. O(log n).
    pub fn first_true(self, ok: (K, V) -> Bool) -> ?(K, V) = todo

    /// Largest entry where monotone `ok(k, v)` still holds, or `none`. O(log n).
    pub fn last_true(self, ok: (K, V) -> Bool) -> ?(K, V) = todo

    /// Remove and return entries after `after` (ascending) or before `before` (descending) while `ok` holds. O(k log n).
    pub fn drain(mut self; after: ?K = none, before: ?K = none, while ok: (K, V) -> Bool) -> List<(K, V)> = todo

    /// Split off and return all entries with key `>= k`. O(log n).
    pub fn split_at(mut self, k: K) -> SortedMap<K, V> = todo

    /// Concatenate `other` onto `self` when every key in `self` is `<` every key in `other`. O(log n).
    pub fn join(mut self, other: SortedMap<K, V>) = todo

    /// Merge all entries of `other` into `self` (emptying `other`), combining collisions with `join` or overwriting. O(min(n, m) log(n + m)).
    pub fn absorb(mut self, mut other: SortedMap<K, V>, join: ?((V, V) -> V) = none) = todo

    /// Yield `(key, val)` pairs in ascending key order. O(n).
    pub fn each(self) -> yields (K, V) = todo

    /// Yield `(key, val)` pairs in descending key order. O(n).
    pub fn each_rev(self) -> yields (K, V) = todo

    /// Yield keys in ascending order. O(n).
    pub fn keys(self) -> yields K = todo

    /// Yield values in ascending key order. O(n).
    pub fn values(self) -> yields V = todo

/// Monoid-augmented ordered multiset maintaining `op` over `leaf(k)` on every subtree (W10 operation values).
/// Supports O(log n) `add`, `remove`, `rank`, `nth`, `fold_lt`, `fold_le`, `fold_ge`, `fold_gt`, `fold_between`, and O(1) `all`.
pub struct AugSortedBag<K, T>:
    _len: Int

    /// Create an empty `AugSortedBag<K, T>` with associative `op`, identity `id`, and key map `leaf`. O(1).
    pub fn new(op: (T, T) -> T, id: T, leaf: K -> T) -> AugSortedBag<K, T> = todo

    /// Preallocate treap node pool for at least `cap` elements. O(cap).
    pub fn reserve(mut self, cap: Int) = todo

    /// Total number of elements (with multiplicity). O(1).
    pub @get fn len(self) -> Int = todo

    /// Whether the bag is empty. O(1).
    pub @get fn is_empty(self) -> Bool = todo

    /// Fold of all elements in the bag. O(1).
    pub @get fn all(self) -> T = todo

    /// Insert one copy of `x`. O(log n).
    pub fn add(mut self, x: K) = todo

    /// Remove one copy of `x`; returns false if absent. O(log n).
    pub fn remove(mut self, x: K) -> Bool = todo

    /// Multiplicity of `x`. O(log n).
    pub fn count(self, x: K) -> Int = todo

    /// Number of elements strictly less than `k`. O(log n).
    pub fn rank(self, k: K) -> Int = todo

    /// The `i`-th smallest element (0-based), or `none`. O(log n).
    pub fn nth(self, i: Int) -> ?K = todo

    /// Smallest element, or `none`. O(1), kept current by every update.
    pub @get fn first(self) -> ?K = todo

    /// Largest element, or `none`. O(1), kept current by every update.
    pub @get fn last(self) -> ?K = todo

    /// Fold elements with key `< hi`. O(log n).
    pub fn fold_lt(self, hi: K) -> T = todo

    /// Fold elements with key `<= hi`. O(log n).
    pub fn fold_le(self, hi: K) -> T = todo

    /// Fold elements with key `>= lo`. O(log n).
    pub fn fold_ge(self, lo: K) -> T = todo

    /// Fold elements with key `> lo`. O(log n).
    pub fn fold_gt(self, lo: K) -> T = todo

    /// Fold elements with key in `[lo, hi)`. O(log n).
    pub fn fold_between(self, lo: K, hi: K) -> T = todo

    /// Yield all elements in ascending order. O(n).
    pub fn each(self) -> yields K = todo

/// Monoid-augmented ordered unique set maintaining `op` over `leaf(k)` on every subtree (W10 operation values).
/// Covers dynamic range sums and consecutive-gap monoids (`cf2066e`) in O(log n).
pub struct AugSortedSet<K, T>:
    _len: Int

    /// Create an empty `AugSortedSet<K, T>` with associative `op`, identity `id`, and key map `leaf`. O(1).
    pub fn new(op: (T, T) -> T, id: T, leaf: K -> T) -> AugSortedSet<K, T> = todo

    /// Preallocate treap node pool for at least `cap` keys. O(cap).
    pub fn reserve(mut self, cap: Int) = todo

    /// Number of distinct keys. O(1).
    pub @get fn len(self) -> Int = todo

    /// Whether the set is empty. O(1).
    pub @get fn is_empty(self) -> Bool = todo

    /// Fold of all keys in the set. O(1).
    pub @get fn all(self) -> T = todo

    /// Insert `x`; returns true if `x` was new. O(log n).
    pub fn add(mut self, x: K) -> Bool = todo

    /// Remove `x`; returns true if `x` was present. O(log n).
    pub fn remove(mut self, x: K) -> Bool = todo

    /// Whether `x` is present. O(log n).
    pub fn contains(self, x: K) -> Bool = todo

    /// Number of keys strictly less than `k`. O(log n).
    pub fn rank(self, k: K) -> Int = todo

    /// The `i`-th smallest key (0-based), or `none`. O(log n).
    pub fn nth(self, i: Int) -> ?K = todo

    /// Smallest key, or `none`. O(1), kept current by every update.
    pub @get fn first(self) -> ?K = todo

    /// Largest key, or `none`. O(1), kept current by every update.
    pub @get fn last(self) -> ?K = todo

    /// Fold keys `< hi`. O(log n).
    pub fn fold_lt(self, hi: K) -> T = todo

    /// Fold keys `<= hi`. O(log n).
    pub fn fold_le(self, hi: K) -> T = todo

    /// Fold keys `>= lo`. O(log n).
    pub fn fold_ge(self, lo: K) -> T = todo

    /// Fold keys `> lo`. O(log n).
    pub fn fold_gt(self, lo: K) -> T = todo

    /// Fold keys in `[lo, hi)`. O(log n).
    pub fn fold_between(self, lo: K, hi: K) -> T = todo

    /// Yield all keys in ascending order. O(n).
    pub fn each(self) -> yields K = todo

/// O(1) point add, O(sqrt(N)) k-th order statistic over an integer value range `r`.
/// Designed as a Mo's algorithm companion for value-range frequency tables.
pub struct BucketCount:
    _lo: Int
    _hi: Int

    /// O(r.len). Create an empty bucketed frequency table over `r`.
    pub fn new(r: Range<Int>) -> BucketCount = todo

    /// O(1). Add `delta` (default 1; may be negative) copies of `x`.
    pub fn add(mut self, x: Int, delta: Int = 1) = todo

    /// O(1). Remove `delta` (default 1) copies of `x`.
    pub fn remove(mut self, x: Int, delta: Int = 1) = todo

    /// O(1). Multiplicity of `x`.
    pub fn count(self, x: Int) -> Int = todo

    /// O(sqrt(N)). Number of elements strictly less than `x`.
    pub fn count_lt(self, x: Int) -> Int = todo

    /// O(sqrt(N)). The `k`-th smallest element (1-based).
    pub fn kth(self, k: Int) -> Int = todo

    /// O(1). Total number of elements (with multiplicity).
    pub @get fn len(self) -> Int = todo

    /// O(1). Whether the table is empty.
    pub @get fn is_empty(self) -> Bool = todo

    /// O(r.len). Reset all counts to zero.
    pub fn clear(mut self) = todo

@test
fn sorted_bag_basics_duplicates_and_take() = todo

@test
fn sorted_bag_matches_sorted_list_on_random_ops() = todo

@test
fn sorted_bag_inversion_count_and_generic_keys() = todo

@test
fn sorted_bag_cursors_entries_and_recycling() = todo

@test
fn sorted_set_operations_range_and_bsearch() = todo

@test
fn sorted_set_and_bag_drain_after_before_and_random() = todo
