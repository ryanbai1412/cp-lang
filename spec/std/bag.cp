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
/// `add`, `add_many`, `remove`, `remove_all`, `contains`, `count`,
/// `pop_first`, `pop_last`, `rank`, `nth`, `first_true`, `last_true`, and `cursor_*` are O(log n) expected;
/// `range(r)` returns an O(1) view whose `.first`, `.last`, `.len`, `.is_empty` are O(log n)
/// expected and whose iteration (`for x in b.range(r)` / `.rev`) is O(log n + k);
/// `first`, `last`, `key_of` and `count_of` are O(1); `next_cursor` and `prev_cursor` are O(1) amortized;
/// `len`, `distinct`, `is_empty`, `clear`, and `take` are O(1);
/// `entries`, `entries_rev`, and `to_list` are O(n).
///
/// Elements are ordered by `K`'s `<`; copies of one key are equal and indistinguishable, so
/// no method here has a tie to break between them. `n` in every cost is `len`.
pub struct SortedBag<K>:
    nodes: List<BagNode<K>>
    free: List<-2_147_483_648..=2_147_483_647>
    live: Int
    root: Int
    rng: U64

    /// Create an empty `SortedBag<K>`. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var b: SortedBag<Int> = SortedBag.new(); b.len == 0`
    pub fn new() -> SortedBag<K> = todo

    /// Build a `SortedBag` containing every element of `xs`. O(n log n) expected.
    ///
    /// Every copy counts: `SortedBag.of([2, 2])` holds two elements.
    ///
    /// cost: O(n log n) expected
    /// empty: an empty bag
    /// traps: never
    /// example: `SortedBag.of([3, 1, 3]).to_list() == [1, 3, 3]`
    pub fn of(xs: List<K>) -> SortedBag<K> = todo

    /// Total number of elements in the bag (counting duplicates). O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `SortedBag.of([3, 1, 3]).len == 3`
    pub @get fn len(self) -> Int = todo

    /// Number of distinct keys currently in the bag. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `SortedBag.of([3, 1, 3]).distinct == 2`
    pub @get fn distinct(self) -> Int = todo

    /// Whether the bag has no elements. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `SortedBag.of([1]).is_empty == false`
    pub @get fn is_empty(self) -> Bool = todo

    /// Remove all elements from the bag. O(1).
    ///
    /// The node array keeps its capacity.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var b = SortedBag.of([1, 2]); b.clear(); b.len == 0 and b.first == None`
    pub fn clear(mut self) = todo

    /// Move the bag's contents out in O(1), leaving `self` empty.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var b = SortedBag.of([1, 2]); let t = b.take(); b.is_empty and t.to_list() == [1, 2]`
    pub fn take(mut self) -> SortedBag<K> = todo

    /// Insert one copy of `x` into the bag. Always returns `true`. O(log n) expected.
    ///
    /// The `Bool` matches `SortedSet.add`, so code can switch between the two.
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var b: SortedBag<Int> = SortedBag.new(); b.add(3) and b.add(3) and b.count(3) == 2`
    pub fn add(mut self, x: K) -> Bool = todo

    fn add_unique(mut self, x: K) -> Bool = todo

    /// Insert `c >= 0` copies of `x` into the bag. O(log n) expected.
    ///
    /// `c == 0` does nothing.
    ///
    /// cost: O(log n) expected, independent of `c`
    /// traps: when `c < 0`
    /// example: `var b = SortedBag.of([1]); b.add_many(5, 3); b.len == 4 and b.count(5) == 3`
    pub fn add_many(mut self, x: K, c: Int) = todo

    /// Remove one copy of `x` if present; returns `true` if a copy was removed. O(log n) expected.
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var b = SortedBag.of([2, 2]); b.remove(2) and b.count(2) == 1 and not b.remove(7)`
    pub fn remove(mut self, x: K) -> Bool = todo

    /// Remove all copies of `x` and return how many were removed. O(log n) expected.
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var b = SortedBag.of([2, 2, 3]); b.remove_all(2) == 2 and b.to_list() == [3]`
    /// example: `var b = SortedBag.of([3]); b.remove_all(2) == 0`
    pub fn remove_all(mut self, x: K) -> Int = todo

    /// How many copies of `x` are currently in the bag. O(log n) expected.
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `SortedBag.of([2, 2, 3]).count(2) == 2`
    /// example: `SortedBag.of([2, 2, 3]).count(4) == 0`
    pub fn count(self, x: K) -> Int = todo

    /// Whether at least one copy of `x` is in the bag. O(log n) expected.
    ///
    /// `x in b` calls this.
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `SortedBag.of([2, 3]).contains(3) == true`
    pub fn contains(self, x: K) -> Bool = todo

    /// The smallest element, or `None` when empty. O(1), kept current by every update.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// ties: none: copies of the least key are equal
    /// traps: never
    /// example: `SortedBag.of([3, 1, 1]).first == Some(1)`
    pub @get fn first(self) -> ?K = todo

    /// The largest element, or `None` when empty. O(1), kept current by every update.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// ties: none: copies of the greatest key are equal
    /// traps: never
    /// example: `SortedBag.of([3, 1, 3]).last == Some(3)`
    pub @get fn last(self) -> ?K = todo

    /// Remove and return one copy of the smallest element, or `None` when empty. O(log n) expected.
    ///
    /// cost: O(log n) expected
    /// empty: `None`
    /// traps: never
    /// example: `var b = SortedBag.of([3, 1, 1]); b.pop_first() == Some(1) and b.count(1) == 1`
    pub fn pop_first(mut self) -> ?K = todo

    /// O(log n). Removes and returns the least element satisfying monotone `ok` (false then
    /// true): "the shortest partner that completes this chain".
    ///
    /// One copy is removed. `None`, and no change, when `ok` holds for no element. It is
    /// `first_true(ok)` followed by `remove` of the result, in one descent.
    ///
    /// cost: O(log n) expected
    /// empty: `None`
    /// ties: none: copies of the answer are equal; one is removed
    /// traps: never in release; debug traps when `ok` is not false-then-true at the answer
    /// example: `var b = SortedBag.of([1, 4, 4, 6]); b.take_first(x => x >= 3) == Some(4) and b.to_list() == [1, 4, 6]`
    /// example: `var b = SortedBag.of([1]); b.take_first(x => x >= 3) == None and b.len == 1`
    pub fn take_first(mut self, ok: K -> Bool) -> ?K = todo

    /// O(log n + k). Removes the elements whose value lies in `r`, yielding them in order.
    ///
    /// Each copy is yielded once, so a key with count `c` is yielded `c` times. A loop that
    /// stops early leaves the elements not yet yielded in the bag. `r` may be any ordered range
    /// (`a..b`, `a..=b`, `..b`, `a..`, `..`).
    ///
    /// cost: O(log n + k) expected for `k` removed
    /// empty: yields nothing, for an empty bag or an empty `r`
    /// order: ascending
    /// traps: never
    /// example: `var b = SortedBag.of([1, 3, 3, 5, 7]); b.drain_range(3..6).collect() == [3, 3, 5] and b.to_list() == [1, 7]`
    pub fn drain_range(mut self, r: Range<K>) -> yields K = todo

    /// Remove and return one copy of the largest element, or `None` when empty. O(log n) expected.
    ///
    /// cost: O(log n) expected
    /// empty: `None`
    /// traps: never
    /// example: `var b = SortedBag.of([3, 3, 1]); b.pop_last() == Some(3) and b.to_list() == [1, 3]`
    pub fn pop_last(mut self) -> ?K = todo

    /// Number of elements strictly less than `k` (counting duplicates). O(log n) expected.
    ///
    /// `k` need not be in the bag. `b.nth(b.rank(k))` is the least element `>= k`.
    ///
    /// cost: O(log n) expected
    /// empty: `0`
    /// traps: never
    /// example: `SortedBag.of([1, 2, 2, 5]).rank(2) == 1`
    /// example: `SortedBag.of([1, 2, 2, 5]).rank(3) == 3`
    pub fn rank(self, k: K) -> Int = todo

    fn rank_le(self, k: K) -> Int = todo

    fn lo_ok(self, x: K, lo: ?K, lo_inc: Bool) -> Bool = todo

    fn hi_ok(self, x: K, hi: ?K, hi_inc: Bool) -> Bool = todo

    fn range_first(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> ?K = todo

    fn range_last(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> ?K = todo

    fn range_len(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> Int = todo

    fn range_each(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> yields K = todo

    fn range_each_rev(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> yields K = todo

    /// 0-indexed `i`-th smallest element (counting duplicates), or `None` if out of bounds. O(log n) expected.
    ///
    /// It is `b.to_list()[i]` without the list; `None` for any `i` outside `0..len`, negative
    /// included.
    ///
    /// cost: O(log n) expected
    /// empty: `None`
    /// traps: never
    /// example: `SortedBag.of([5, 2, 2, 1]).nth(2) == Some(2)`
    /// example: `SortedBag.of([5, 2, 2, 1]).nth(4) == None`
    pub fn nth(self, i: Int) -> ?K = todo

    /// Smallest key where monotone `ok(k)` holds (`false..false, true..true`), or `None`.
    /// Checks monotonicity at the boundary. O(log n) expected.
    ///
    /// `ok` is called on keys along one root-to-leaf path. Debug builds check that `ok` is
    /// false on the key just below the answer.
    ///
    /// cost: O(log n) expected calls of `ok`
    /// empty: `None`
    /// ties: none: copies of a key are one candidate; the least key wins
    /// traps: never in release; debug traps when `ok` is not false-then-true at the answer
    /// example: `SortedBag.of([1, 4, 6]).first_true(x => x >= 3) == Some(4)`
    /// example: `SortedBag.of([1, 4, 6]).first_true(x => x > 9) == None`
    pub fn first_true(self, ok: K -> Bool) -> ?K = todo

    /// Largest key where monotone `ok(k)` holds (`true..true, false..false`), or `None`.
    /// Checks monotonicity at the boundary. O(log n) expected.
    ///
    /// cost: O(log n) expected calls of `ok`
    /// empty: `None`
    /// ties: none: copies of a key are one candidate; the greatest key wins
    /// traps: never in release; debug traps when `ok` is not true-then-false at the answer
    /// example: `SortedBag.of([1, 4, 6]).last_true(x => x <= 5) == Some(4)`
    /// example: `SortedBag.of([1, 4, 6]).last_true(x => x < 1) == None`
    pub fn last_true(self, ok: K -> Bool) -> ?K = todo

    /// Remove and return the maximal run of elements strictly after `after` (ascending),
    /// strictly before `before` (descending), or from the smallest element (ascending)
    /// while `ok(k)` holds. O(k log n) expected for `k` removed elements.
    ///
    /// The walk starts at the first element past the bound and stops at the first element
    /// where `ok` is false, which stays; each copy of a removed key is one element of the
    /// result. Giving both `after` and `before` is an error.
    ///
    /// cost: O(k log n) expected for `k` removed elements
    /// empty: `[]`
    /// order: ascending with `after` or no bound, descending with `before`; a new `List`, keyed from 0
    /// traps: when both `after` and `before` are given
    /// example: `var b = SortedBag.of([1, 2, 2, 5, 8]); b.drain(after: 1, while: x => x < 5) == [2, 2] and b.to_list() == [1, 5, 8]`
    /// example: `var b = SortedBag.of([1, 2, 5, 8]); b.drain(before: 8, while: x => x > 1) == [5, 2] and b.to_list() == [1, 8]`
    /// example: `var b = SortedBag.of([1, 2, 5]); b.drain(while: x => x < 3) == [1, 2]`
    pub fn drain(mut self; after: ?K = None, before: ?K = None, while ok: K -> Bool) -> List<K> = todo

    /// Cursor to the smallest key, or `None` when empty. O(log n) expected.
    ///
    /// cost: O(log n) expected
    /// empty: `None`
    /// traps: never
    /// example: `let b = SortedBag.of([5, 2, 2]); b.key_of(b.cursor_first()!) == 2`
    pub fn cursor_first(self) -> ?BagCursor = todo

    /// Cursor to the largest key, or `None` when empty. O(log n) expected.
    ///
    /// cost: O(log n) expected
    /// empty: `None`
    /// traps: never
    /// example: `let b = SortedBag.of([5, 2, 2]); b.key_of(b.cursor_last()!) == 5`
    pub fn cursor_last(self) -> ?BagCursor = todo

    /// Cursor to exact key `k`, or `None` if absent. O(log n) expected.
    ///
    /// cost: O(log n) expected
    /// empty: `None`
    /// traps: never
    /// example: `let b = SortedBag.of([2, 5]); b.cursor_at(5) != None and b.cursor_at(3) == None`
    pub fn cursor_at(self, k: K) -> ?BagCursor = todo

    /// Cursor to smallest key `>= k`, or `None`. O(log n) expected.
    ///
    /// cost: O(log n) expected
    /// empty: `None`
    /// traps: never
    /// example: `let b = SortedBag.of([2, 5]); b.key_of(b.cursor_ge(3)!) == 5 and b.cursor_ge(6) == None`
    pub fn cursor_ge(self, k: K) -> ?BagCursor = todo

    /// Cursor to smallest key `> k`, or `None`. O(log n) expected.
    ///
    /// cost: O(log n) expected
    /// empty: `None`
    /// traps: never
    /// example: `let b = SortedBag.of([2, 5]); b.key_of(b.cursor_gt(2)!) == 5 and b.cursor_gt(5) == None`
    pub fn cursor_gt(self, k: K) -> ?BagCursor = todo

    /// Cursor to largest key `<= k`, or `None`. O(log n) expected.
    ///
    /// cost: O(log n) expected
    /// empty: `None`
    /// traps: never
    /// example: `let b = SortedBag.of([2, 5]); b.key_of(b.cursor_le(4)!) == 2 and b.cursor_le(1) == None`
    pub fn cursor_le(self, k: K) -> ?BagCursor = todo

    /// Cursor to largest key `< k`, or `None`. O(log n) expected.
    ///
    /// cost: O(log n) expected
    /// empty: `None`
    /// traps: never
    /// example: `let b = SortedBag.of([2, 5]); b.key_of(b.cursor_lt(5)!) == 2 and b.cursor_lt(2) == None`
    pub fn cursor_lt(self, k: K) -> ?BagCursor = todo

    /// Key stored at cursor `c`. O(1).
    ///
    /// `c` must come from this bag and no mutation may have happened since it was made.
    ///
    /// cost: O(1)
    /// traps: never in release (a stale cursor reads an unspecified key); debug traps when the node of `c` has been freed
    /// example: `let b = SortedBag.of([2, 2, 5]); b.key_of(b.cursor_first()!) == 2`
    pub fn key_of(self, c: BagCursor) -> K = todo

    /// Multiplicity stored at cursor `c`. O(1).
    ///
    /// cost: O(1)
    /// traps: never in release (a stale cursor reads an unspecified count); debug traps when the node of `c` has been freed
    /// example: `let b = SortedBag.of([2, 2, 5]); b.count_of(b.cursor_at(2)!) == 2`
    pub fn count_of(self, c: BagCursor) -> Int = todo

    /// Cursor to the next larger distinct key, or `None`. O(1) amortized.
    ///
    /// Steps over all copies of the current key at once.
    ///
    /// cost: O(1) amortized, O(log n) worst case
    /// traps: never in release; debug traps when the node of `c` has been freed
    /// example: `let b = SortedBag.of([2, 2, 5]); b.key_of(b.next_cursor(b.cursor_first()!)!) == 5`
    /// example: `let b = SortedBag.of([2, 5]); b.next_cursor(b.cursor_last()!) == None`
    pub fn next_cursor(self, c: BagCursor) -> ?BagCursor = todo

    /// Cursor to the next smaller distinct key, or `None`. O(1) amortized.
    ///
    /// cost: O(1) amortized, O(log n) worst case
    /// traps: never in release; debug traps when the node of `c` has been freed
    /// example: `let b = SortedBag.of([2, 5, 5]); b.key_of(b.prev_cursor(b.cursor_last()!)!) == 2`
    /// example: `let b = SortedBag.of([2, 5]); b.prev_cursor(b.cursor_first()!) == None`
    pub fn prev_cursor(self, c: BagCursor) -> ?BagCursor = todo

    /// Visit every distinct `(key, count)` in ascending order. O(1) amortized per step.
    ///
    /// cost: O(1) amortized per step
    /// empty: yields nothing
    /// order: ascending key, one entry per distinct key
    /// traps: never
    /// example: `[e.key for e in SortedBag.of([5, 2, 2]).entries()] == [2, 5]`
    /// example: `[e.count for e in SortedBag.of([5, 2, 2]).entries()] == [2, 1]`
    pub fn entries(self) -> yields (key: K, count: Int) = todo

    /// Visit every distinct `(key, count)` in descending order. O(1) amortized per step.
    ///
    /// Exactly the reverse of `entries`.
    ///
    /// cost: O(1) amortized per step
    /// empty: yields nothing
    /// order: descending key, one entry per distinct key
    /// traps: never
    /// example: `[e.key for e in SortedBag.of([5, 2, 2]).entries_rev()] == [5, 2]`
    pub fn entries_rev(self) -> yields (key: K, count: Int) = todo

    /// Every element in ascending order, repeated by multiplicity. O(n).
    ///
    /// cost: O(n)
    /// empty: `[]`
    /// order: ascending, each key repeated by its count; a new `List`, keyed from 0
    /// traps: never
    /// example: `SortedBag.of([3, 1, 3]).to_list() == [1, 3, 3]`
    pub fn to_list(self) -> List<K> = todo

    /// Preallocate treap node pool for at least `cap` distinct keys. O(cap).
    ///
    /// cost: O(cap) when the pool grows, O(1) otherwise
    /// traps: never
    /// example: `var b = SortedBag.of([1]); b.reserve(100); b.to_list() == [1]`
    pub fn reserve(mut self, cap: Int) = todo

    /// Split off and return all elements `>= k`, keeping `< k` in `self`. O(log n).
    ///
    /// Every copy of a key goes to the same side.
    ///
    /// cost: O(log n) expected
    /// empty: both sides empty
    /// traps: never
    /// example: `var b = SortedBag.of([1, 3, 3, 5]); let hi = b.split_at(3); b.to_list() == [1] and hi.to_list() == [3, 3, 5]`
    pub fn split_at(mut self, k: K) -> SortedBag<K> = todo

    /// Concatenate `other` onto `self` when every key in `self` is `<` every key in `other`. O(log n).
    ///
    /// The inverse of `split_at`. The precondition is checked in O(1) against `self.last` and
    /// `other.first`; either side may be empty.
    ///
    /// cost: O(log n) expected
    /// traps: when both are non-empty and `self.last >= other.first`
    /// example: `var a = SortedBag.of([1, 2]); a.join(SortedBag.of([4, 4])); a.to_list() == [1, 2, 4, 4]`
    pub fn join(mut self, other: SortedBag<K>) = todo

    /// Merge all elements of `other` into `self` (small-to-large, emptying `other`). O(min(n, m) log(n + m)).
    ///
    /// Copies add up: a key with count `a` in `self` and `b` in `other` ends with `a + b`. The
    /// smaller bag's elements are inserted into the larger, whichever side that is.
    ///
    /// cost: O(min(n, m) log(n + m)) expected
    /// empty: an empty `other` leaves `self` unchanged
    /// traps: never
    /// example: `var a = SortedBag.of([1, 3]); var b = SortedBag.of([3, 2]); a.absorb(mut b); a.to_list() == [1, 2, 3, 3] and b.is_empty`
    pub fn absorb(mut self, mut other: SortedBag<K>) = todo

    /// Sum of all elements (for additive `K`). O(1) when augmented, O(n) otherwise.
    ///
    /// Each copy is added. `BagNode` keeps no subtree sum, so this bag is never augmented and
    /// the cost is O(n); `AugSortedBag` keeps sums.
    ///
    /// cost: O(n)
    /// empty: `K.zero`
    /// traps: never (overflow wraps in release, traps in debug)
    /// example: `SortedBag.of([2, 2, 5]).sum() == 9`
    pub fn sum(self) -> K = todo

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
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var s: SortedSet<Int> = SortedSet.new(); s.len == 0`
    pub fn new() -> SortedSet<K> = todo

    /// Build a `SortedSet` containing the distinct elements of `xs`. O(n log n) expected.
    ///
    /// cost: O(n log n) expected
    /// empty: an empty set
    /// traps: never
    /// example: `SortedSet.of([3, 1, 3]).to_list() == [1, 3]`
    pub fn of(xs: List<K>) -> SortedSet<K> = todo

    /// Number of keys in the set. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `SortedSet.of([3, 1, 3]).len == 2`
    pub @get fn len(self) -> Int = todo

    /// Whether the set is empty. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `SortedSet.of([1]).is_empty == false`
    pub @get fn is_empty(self) -> Bool = todo

    /// Remove all keys from the set. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var s = SortedSet.of([1, 2]); s.clear(); s.len == 0`
    pub fn clear(mut self) = todo

    /// Move all keys out, leaving `self` empty. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var s = SortedSet.of([1, 2]); let t = s.take(); s.is_empty and t.to_list() == [1, 2]`
    pub fn take(mut self) -> SortedSet<K> = todo

    /// Insert `x` if absent; returns `true` when `x` was newly inserted. O(log n) expected.
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var s = SortedSet.of([1]); s.add(2) and not s.add(2) and s.len == 2`
    pub fn add(mut self, x: K) -> Bool = todo

    /// Remove `x` if present; returns `true` if removed. O(log n) expected.
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var s = SortedSet.of([1, 2]); s.remove(2) and not s.remove(2) and s.to_list() == [1]`
    pub fn remove(mut self, x: K) -> Bool = todo

    /// Add `x` when `on` is true, or remove it otherwise, matching `Set.set` and `Bitset.set`. O(log n) expected.
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var s = SortedSet.of([1]); s.set(2, true); s.set(1, false); s.to_list() == [2]`
    pub fn set(mut self, x: K, on: Bool) = todo

    /// Whether `x` is in the set. O(log n) expected.
    ///
    /// `x in s` calls this.
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `SortedSet.of([1, 4]).contains(4) and not SortedSet.of([1, 4]).contains(2)`
    pub fn contains(self, x: K) -> Bool = todo

    /// Smallest key in the set, or `None`. O(1), kept current by every update.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// ties: none: keys are distinct
    /// traps: never
    /// example: `SortedSet.of([4, 1, 9]).first == Some(1)`
    pub @get fn first(self) -> ?K = todo

    /// Largest key in the set, or `None`. O(1), kept current by every update.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// ties: none: keys are distinct
    /// traps: never
    /// example: `SortedSet.of([4, 1, 9]).last == Some(9)`
    pub @get fn last(self) -> ?K = todo

    /// Remove and return the smallest key, or `None`. O(log n) expected.
    ///
    /// cost: O(log n) expected
    /// empty: `None`
    /// traps: never
    /// example: `var s = SortedSet.of([4, 1]); s.pop_first() == Some(1) and s.to_list() == [4]`
    pub fn pop_first(mut self) -> ?K = todo

    /// Remove and return the largest key, or `None`. O(log n) expected.
    ///
    /// cost: O(log n) expected
    /// empty: `None`
    /// traps: never
    /// example: `var s = SortedSet.of([4, 1]); s.pop_last() == Some(4) and s.to_list() == [1]`
    pub fn pop_last(mut self) -> ?K = todo

    /// Number of keys strictly less than `k`. O(log n) expected.
    ///
    /// `k` need not be in the set.
    ///
    /// cost: O(log n) expected
    /// empty: `0`
    /// traps: never
    /// example: `SortedSet.of([1, 4, 9]).rank(4) == 1`
    /// example: `SortedSet.of([1, 4, 9]).rank(5) == 2`
    pub fn rank(self, k: K) -> Int = todo

    /// 0-indexed `i`-th smallest key, or `None` if out of range. O(log n) expected.
    ///
    /// `None` for any `i` outside `0..len`, negative included.
    ///
    /// cost: O(log n) expected
    /// empty: `None`
    /// traps: never
    /// example: `SortedSet.of([9, 1, 4]).nth(1) == Some(4)`
    /// example: `SortedSet.of([9, 1, 4]).nth(-1) == None`
    pub fn nth(self, i: Int) -> ?K = todo

    /// The sum of the elements in `keys` (all of them by default), for additive `K`:
    /// `set.sum(..b)` adds everything below `b`. O(log n): subtree sums are kept.
    ///
    /// `keys` may be any ordered range (`a..b`, `a..=b`, `..b`, `a..`, `..`); its ends need
    /// not be in the set.
    ///
    /// cost: O(log n) expected
    /// empty: `K.zero`, for an empty set or an empty `keys`
    /// traps: never (overflow wraps in release, traps in debug)
    /// example: `SortedSet.of([1, 2, 5]).sum() == 8`
    /// example: `SortedSet.of([1, 2, 5]).sum(..3) == 3`
    /// example: `SortedSet.of([1, 2, 5]).sum(2..=5) == 7`
    pub fn sum(self, keys: Range<K> = ..) -> K = todo

    /// Smallest key `k` where monotone `ok(k)` holds (`false..false, true..true`), or `None`. O(log n) expected.
    ///
    /// cost: O(log n) expected calls of `ok`
    /// empty: `None`
    /// ties: none: keys are distinct; the least key wins
    /// traps: never in release; debug traps when `ok` is not false-then-true at the answer
    /// example: `SortedSet.of([1, 4, 6]).first_true(x => x >= 3) == Some(4)`
    pub fn first_true(self, ok: K -> Bool) -> ?K = todo

    /// Largest key `k` where monotone `ok(k)` holds (`true..true, false..false`), or `None`. O(log n) expected.
    ///
    /// cost: O(log n) expected calls of `ok`
    /// empty: `None`
    /// ties: none: keys are distinct; the greatest key wins
    /// traps: never in release; debug traps when `ok` is not true-then-false at the answer
    /// example: `SortedSet.of([1, 4, 6]).last_true(x => x <= 5) == Some(4)`
    pub fn last_true(self, ok: K -> Bool) -> ?K = todo

    /// Remove and return the maximal run of keys strictly after `after` (ascending),
    /// strictly before `before` (descending), or from the smallest key (ascending)
    /// while `ok(k)` holds. O(k log n) expected for `k` removed keys.
    ///
    /// The walk stops at the first key where `ok` is false, which stays. Giving both `after`
    /// and `before` is an error.
    ///
    /// cost: O(k log n) expected for `k` removed keys
    /// empty: `[]`
    /// order: ascending with `after` or no bound, descending with `before`; a new `List`, keyed from 0
    /// traps: when both `after` and `before` are given
    /// example: `var s = SortedSet.of([1, 2, 5, 8]); s.drain(after: 1, while: x => x < 6) == [2, 5] and s.to_list() == [1, 8]`
    /// example: `var s = SortedSet.of([1, 2, 5, 8]); s.drain(before: 8, while: x => x > 1) == [5, 2]`
    pub fn drain(mut self; after: ?K = None, before: ?K = None, while ok: K -> Bool) -> List<K> = todo

    fn range_first(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> ?K = todo

    fn range_last(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> ?K = todo

    fn range_len(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> Int = todo

    fn range_each(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> yields K = todo

    fn range_each_rev(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> yields K = todo

    /// Visit every key in ascending order. O(1) amortized per step.
    ///
    /// `for x in s` uses this.
    ///
    /// cost: O(1) amortized per step
    /// empty: yields nothing
    /// order: ascending
    /// traps: never
    /// example: `SortedSet.of([3, 1, 2]).each().collect() == [1, 2, 3]`
    pub fn each(self) -> yields K = todo

    /// Visit every key in descending order. O(1) amortized per step.
    ///
    /// cost: O(1) amortized per step
    /// empty: yields nothing
    /// order: descending
    /// traps: never
    /// example: `SortedSet.of([3, 1, 2]).each_rev().collect() == [3, 2, 1]`
    pub fn each_rev(self) -> yields K = todo

    /// Return all keys in ascending order as a `List<K>`. O(n).
    ///
    /// cost: O(n)
    /// empty: `[]`
    /// order: ascending; a new `List`, keyed from 0
    /// traps: never
    /// example: `SortedSet.of([3, 1, 2]).to_list() == [1, 2, 3]`
    pub fn to_list(self) -> List<K> = todo

    /// Preallocate treap node pool for at least `cap` keys. O(cap).
    ///
    /// cost: O(cap) when the pool grows, O(1) otherwise
    /// traps: never
    /// example: `var s = SortedSet.of([1]); s.reserve(100); s.to_list() == [1]`
    pub fn reserve(mut self, cap: Int) = todo

    /// Split off and return all keys `>= k`, keeping `< k` in `self`. O(log n).
    ///
    /// cost: O(log n) expected
    /// empty: both sides empty
    /// traps: never
    /// example: `var s = SortedSet.of([1, 3, 5]); let hi = s.split_at(3); s.to_list() == [1] and hi.to_list() == [3, 5]`
    pub fn split_at(mut self, k: K) -> SortedSet<K> = todo

    /// Concatenate `other` onto `self` when every key in `self` is `<` every key in `other`. O(log n).
    ///
    /// The precondition is checked in O(1) against `self.last` and `other.first`.
    ///
    /// cost: O(log n) expected
    /// traps: when both are non-empty and `self.last >= other.first`
    /// example: `var a = SortedSet.of([1, 2]); a.join(SortedSet.of([4])); a.to_list() == [1, 2, 4]`
    pub fn join(mut self, other: SortedSet<K>) = todo

    /// Merge all keys of `other` into `self` (small-to-large, emptying `other`). O(min(n, m) log(n + m)).
    ///
    /// A key in both sets appears once.
    ///
    /// cost: O(min(n, m) log(n + m)) expected
    /// empty: an empty `other` leaves `self` unchanged
    /// traps: never
    /// example: `var a = SortedSet.of([1, 3]); var b = SortedSet.of([3, 2]); a.absorb(mut b); a.to_list() == [1, 2, 3] and b.is_empty`
    pub fn absorb(mut self, mut other: SortedSet<K>) = todo

/// Order-statistic ordered map from `K` to `V` backed by a randomized treap.
/// `get`, `set`, `remove`, `rank`, `nth`, `first_ge`/`last_le`, `first_true`/`last_true`, `split_at`, `join` are O(log n).
///
/// Entries are ordered by key; keys are distinct, so no lookup has a tie to break. Every
/// entry-returning method gives `(key, val)`.
pub struct SortedMap<K, V>:
    _len: Int

    /// Create an empty `SortedMap<K, V>`. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap.new(); m.len == 0`
    pub fn new() -> SortedMap<K, V> = todo

    /// Preallocate treap node pool for at least `cap` entries. O(cap).
    ///
    /// cost: O(cap) when the pool grows, O(1) otherwise
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap.new(); m.reserve(100); m.is_empty`
    pub fn reserve(mut self, cap: Int) = todo

    /// Number of key-value pairs. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap.new(); m.set(1, "a"); m.set(1, "b"); m.len == 1`
    pub @get fn len(self) -> Int = todo

    /// Whether the map is empty. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap.new(); m.is_empty`
    pub @get fn is_empty(self) -> Bool = todo

    /// Remove all entries. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap.new(); m.set(1, "a"); m.clear(); m.get(1) == None`
    pub fn clear(mut self) = todo

    /// Move contents out in O(1), leaving `self` empty.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap.new(); m.set(1, "a"); let t = m.take(); m.is_empty and t.get(1) == Some("a")`
    pub fn take(mut self) -> SortedMap<K, V> = todo

    /// Look up `k`, returning `None` if absent. O(log n).
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap.new(); m.set(1, "a"); m.get(1) == Some("a") and m.get(2) == None`
    pub fn get(self, k: K) -> ?V = todo

    /// Whether `k` is present. O(log n).
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap.new(); m.set(1, "a"); m.contains(1) and not m.contains(2)`
    pub fn contains(self, k: K) -> Bool = todo

    /// Insert or overwrite `k -> v` (W8 `IndexSet` backing `m[k] = v`). O(log n).
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap.new(); m[1] = "a"; m[1] = "b"; m.get(1) == Some("b")`
    pub fn set(mut self, k: K, v: V) = todo

    /// Remove `k` and return its previous value, or `None` if absent. O(log n).
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap.new(); m.set(1, "a"); m.remove(1) == Some("a") and m.remove(1) == None`
    pub fn remove(mut self, k: K) -> ?V = todo

    /// Smallest `(key, val)` pair, or `None` if empty. O(1), kept current by every update.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// ties: none: keys are distinct
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap.new(); m.set(3, "c"); m.set(1, "a"); m.first == Some((1, "a"))`
    pub @get fn first(self) -> ?(K, V) = todo

    /// Largest `(key, val)` pair, or `None` if empty. O(1), kept current by every update.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// ties: none: keys are distinct
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap.new(); m.set(3, "c"); m.set(1, "a"); m.last == Some((3, "c"))`
    pub @get fn last(self) -> ?(K, V) = todo

    /// Remove and return the smallest `(key, val)` pair, or `None`. O(log n).
    ///
    /// cost: O(log n) expected
    /// empty: `None`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap.new(); m.set(3, "c"); m.set(1, "a"); m.pop_first() == Some((1, "a")) and m.len == 1`
    pub fn pop_first(mut self) -> ?(K, V) = todo

    /// Remove and return the largest `(key, val)` pair, or `None`. O(log n).
    ///
    /// cost: O(log n) expected
    /// empty: `None`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap.new(); m.set(3, "c"); m.set(1, "a"); m.pop_last() == Some((3, "c")) and m.len == 1`
    pub fn pop_last(mut self) -> ?(K, V) = todo

    /// Smallest entry with key `>= k`, or `None`. O(log n).
    ///
    /// cost: O(log n) expected
    /// empty: `None`
    /// ties: none: keys are distinct
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap.new(); m.set(1, "a"); m.set(3, "c"); m.first_ge(2) == Some((3, "c")) and m.first_ge(3) == Some((3, "c"))`
    pub fn first_ge(self, k: K) -> ?(K, V) = todo

    /// Smallest entry with key `> k`, or `None`. O(log n).
    ///
    /// cost: O(log n) expected
    /// empty: `None`
    /// ties: none: keys are distinct
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap.new(); m.set(1, "a"); m.set(3, "c"); m.first_gt(1) == Some((3, "c")) and m.first_gt(3) == None`
    pub fn first_gt(self, k: K) -> ?(K, V) = todo

    /// Largest entry with key `<= k`, or `None`. O(log n).
    ///
    /// cost: O(log n) expected
    /// empty: `None`
    /// ties: none: keys are distinct
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap.new(); m.set(1, "a"); m.set(3, "c"); m.last_le(2) == Some((1, "a")) and m.last_le(0) == None`
    pub fn last_le(self, k: K) -> ?(K, V) = todo

    /// Largest entry with key `< k`, or `None`. O(log n).
    ///
    /// cost: O(log n) expected
    /// empty: `None`
    /// ties: none: keys are distinct
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap.new(); m.set(1, "a"); m.set(3, "c"); m.last_lt(3) == Some((1, "a")) and m.last_lt(1) == None`
    pub fn last_lt(self, k: K) -> ?(K, V) = todo

    /// Number of entries with key strictly less than `k`. O(log n).
    ///
    /// cost: O(log n) expected
    /// empty: `0`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap.new(); m.set(1, "a"); m.set(3, "c"); m.rank(3) == 1 and m.rank(9) == 2`
    pub fn rank(self, k: K) -> Int = todo

    /// The `i`-th entry in key order (0-based), or `None`. O(log n).
    ///
    /// `None` for any `i` outside `0..len`, negative included.
    ///
    /// cost: O(log n) expected
    /// empty: `None`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap.new(); m.set(3, "c"); m.set(1, "a"); m.nth(1) == Some((3, "c")) and m.nth(2) == None`
    pub fn nth(self, i: Int) -> ?(K, V) = todo

    /// Smallest entry where monotone `ok(k, v)` becomes true, or `None`. O(log n).
    ///
    /// `ok` must be false-then-true along ascending keys.
    ///
    /// cost: O(log n) expected calls of `ok`
    /// empty: `None`
    /// ties: none: keys are distinct; the least key wins
    /// traps: never in release; debug traps when `ok` is not false-then-true at the answer
    /// example: `var m: SortedMap<Int, Int> = SortedMap.new(); m.set(1, 10); m.set(4, 40); m.set(6, 60); m.first_true((k, v) => v >= 30) == Some((4, 40))`
    pub fn first_true(self, ok: (K, V) -> Bool) -> ?(K, V) = todo

    /// Largest entry where monotone `ok(k, v)` still holds, or `None`. O(log n).
    ///
    /// `ok` must be true-then-false along ascending keys.
    ///
    /// cost: O(log n) expected calls of `ok`
    /// empty: `None`
    /// ties: none: keys are distinct; the greatest key wins
    /// traps: never in release; debug traps when `ok` is not true-then-false at the answer
    /// example: `var m: SortedMap<Int, Int> = SortedMap.new(); m.set(1, 10); m.set(4, 40); m.set(6, 60); m.last_true((k, v) => k <= 5) == Some((4, 40))`
    pub fn last_true(self, ok: (K, V) -> Bool) -> ?(K, V) = todo

    /// Remove and return entries after `after` (ascending) or before `before` (descending) while `ok` holds. O(k log n).
    ///
    /// With neither bound the walk starts at the smallest key. It stops at the first entry
    /// where `ok` is false, which stays; bounds are exclusive. Giving both is an error.
    ///
    /// cost: O(k log n) expected for `k` removed entries
    /// empty: `[]`
    /// order: ascending key with `after` or no bound, descending with `before`; a new `List`, keyed from 0
    /// traps: when both `after` and `before` are given
    /// example: `var m: SortedMap<Int, Int> = SortedMap.new(); m.set(1, 10); m.set(4, 40); m.set(6, 60); m.drain(after: 1, while: (k, v) => v < 50) == [(4, 40)] and m.len == 2`
    pub fn drain(mut self; after: ?K = None, before: ?K = None, while ok: (K, V) -> Bool) -> List<(K, V)> = todo

    /// Split off and return all entries with key `>= k`. O(log n).
    ///
    /// `self` keeps the entries with key `< k`.
    ///
    /// cost: O(log n) expected
    /// empty: both sides empty
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap.new(); m.set(1, "a"); m.set(3, "c"); let hi = m.split_at(2); m.len == 1 and hi.first == Some((3, "c"))`
    pub fn split_at(mut self, k: K) -> SortedMap<K, V> = todo

    /// Concatenate `other` onto `self` when every key in `self` is `<` every key in `other`. O(log n).
    ///
    /// The precondition is checked in O(1) against the last key of `self` and the first of
    /// `other`.
    ///
    /// cost: O(log n) expected
    /// traps: when both are non-empty and the last key of `self` is `>=` the first key of `other`
    /// example: `var a: SortedMap<Int, Str> = SortedMap.new(); a.set(1, "a"); var b: SortedMap<Int, Str> = SortedMap.new(); b.set(3, "c"); a.join(b); a.len == 2`
    pub fn join(mut self, other: SortedMap<K, V>) = todo

    /// Merge all entries of `other` into `self` (emptying `other`), combining collisions with `join` or overwriting. O(min(n, m) log(n + m)).
    ///
    /// For a key in both maps the result is `join(self's value, other's value)`, or `other`'s
    /// value when `join` is `None`, whichever map is drained into the other.
    ///
    /// cost: O(min(n, m) log(n + m)) expected
    /// empty: an empty `other` leaves `self` unchanged
    /// traps: never
    /// example: `var a: SortedMap<Int, Int> = SortedMap.new(); a.set(1, 10); var b: SortedMap<Int, Int> = SortedMap.new(); b.set(1, 5); b.set(2, 7); a.absorb(mut b, join: (x, y) => x + y); a.get(1) == Some(15) and a.get(2) == Some(7) and b.is_empty`
    pub fn absorb(mut self, mut other: SortedMap<K, V>, join: ?((V, V) -> V) = None) = todo

    /// O(1) to make. Yields `(key, val)` pairs in ascending key order. O(n).
    ///
    /// cost: O(1) to make, O(1) amortized per step
    /// empty: yields nothing
    /// order: ascending key
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap.new(); m.set(3, "c"); m.set(1, "a"); m.items.collect() == [(1, "a"), (3, "c")]`
    pub @get fn items(self) -> yields (K, V) = todo

    /// Yield `(key, val)` pairs in descending key order. O(n).
    ///
    /// cost: O(1) amortized per step
    /// empty: yields nothing
    /// order: descending key
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap.new(); m.set(3, "c"); m.set(1, "a"); m.each_rev().collect() == [(3, "c"), (1, "a")]`
    pub fn each_rev(self) -> yields (K, V) = todo

    /// O(1) to make. Yields keys in ascending order. O(n).
    ///
    /// cost: O(1) to make, O(1) amortized per step
    /// empty: yields nothing
    /// order: ascending key
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap.new(); m.set(3, "c"); m.set(1, "a"); m.keys.collect() == [1, 3]`
    pub @get fn keys(self) -> yields K = todo

    /// O(1) to make. Yields values in ascending key order. O(n).
    ///
    /// cost: O(1) to make, O(1) amortized per step
    /// empty: yields nothing
    /// order: ascending key
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap.new(); m.set(3, "c"); m.set(1, "a"); m.values.collect() == ["a", "c"]`
    pub @get fn values(self) -> yields V = todo

/// Monoid-augmented ordered multiset maintaining `op` over `leaf(k)` on every subtree (W10 operation values).
/// Supports O(log n) `add`, `remove`, `rank`, `nth`, `fold_lt`, `fold_le`, `fold_ge`, `fold_gt`, `fold_between`, and O(1) `all`.
///
/// Every fold combines `leaf(k)` over the chosen elements in ascending key order, one `leaf(k)`
/// per copy, starting from `id`; `op` need not be commutative. An empty selection folds to `id`.
pub struct AugSortedBag<K, T>:
    _len: Int

    /// Create an empty `AugSortedBag<K, T>` with associative `op`, identity `id`, and key map `leaf`. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag.new((x, y) => x + y, 0, k => k); b.all == 0`
    pub fn new(op: (T, T) -> T, id: T, leaf: K -> T) -> AugSortedBag<K, T> = todo

    /// Preallocate treap node pool for at least `cap` elements. O(cap).
    ///
    /// cost: O(cap) when the pool grows, O(1) otherwise
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag.new((x, y) => x + y, 0, k => k); b.reserve(100); b.len == 0`
    pub fn reserve(mut self, cap: Int) = todo

    /// Total number of elements (with multiplicity). O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag.new((x, y) => x + y, 0, k => k); b.add(3); b.add(3); b.len == 2`
    pub @get fn len(self) -> Int = todo

    /// Whether the bag is empty. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag.new((x, y) => x + y, 0, k => k); b.is_empty`
    pub @get fn is_empty(self) -> Bool = todo

    /// Fold of all elements in the bag. O(1).
    ///
    /// cost: O(1)
    /// empty: `id`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag.new((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.all == 7`
    pub @get fn all(self) -> T = todo

    /// Insert one copy of `x`. O(log n).
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag.new((x, y) => x + y, 0, k => k); b.add(3); b.add(3); b.count(3) == 2`
    pub fn add(mut self, x: K) = todo

    /// Remove one copy of `x`; returns false if absent. O(log n).
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag.new((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.remove(3) and b.all == 4 and not b.remove(2)`
    pub fn remove(mut self, x: K) -> Bool = todo

    /// Multiplicity of `x`. O(log n).
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag.new((x, y) => x + y, 0, k => k); b.add(3); b.add(3); b.count(3) == 2 and b.count(1) == 0`
    pub fn count(self, x: K) -> Int = todo

    /// Number of elements strictly less than `k`. O(log n).
    ///
    /// cost: O(log n) expected
    /// empty: `0`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag.new((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.rank(3) == 1`
    pub fn rank(self, k: K) -> Int = todo

    /// The `i`-th smallest element (0-based), or `None`. O(log n).
    ///
    /// Copies count; `None` for any `i` outside `0..len`.
    ///
    /// cost: O(log n) expected
    /// empty: `None`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag.new((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.nth(2) == Some(3) and b.nth(3) == None`
    pub fn nth(self, i: Int) -> ?K = todo

    /// Smallest element, or `None`. O(1), kept current by every update.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// ties: none: copies of the least key are equal
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag.new((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.first == Some(1)`
    pub @get fn first(self) -> ?K = todo

    /// Largest element, or `None`. O(1), kept current by every update.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// ties: none: copies of the greatest key are equal
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag.new((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.last == Some(3)`
    pub @get fn last(self) -> ?K = todo

    /// Fold elements with key `< hi`. O(log n).
    ///
    /// cost: O(log n) expected
    /// empty: `id`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag.new((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.fold_lt(3) == 1`
    pub fn fold_lt(self, hi: K) -> T = todo

    /// Fold elements with key `<= hi`. O(log n).
    ///
    /// cost: O(log n) expected
    /// empty: `id`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag.new((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.fold_le(3) == 7`
    pub fn fold_le(self, hi: K) -> T = todo

    /// Fold elements with key `>= lo`. O(log n).
    ///
    /// cost: O(log n) expected
    /// empty: `id`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag.new((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.fold_ge(2) == 6`
    pub fn fold_ge(self, lo: K) -> T = todo

    /// Fold elements with key `> lo`. O(log n).
    ///
    /// cost: O(log n) expected
    /// empty: `id`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag.new((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.fold_gt(1) == 6 and b.fold_gt(3) == 0`
    pub fn fold_gt(self, lo: K) -> T = todo

    /// Fold elements with key in `[lo, hi)`. O(log n).
    ///
    /// `lo >= hi` selects nothing and gives `id`.
    ///
    /// cost: O(log n) expected
    /// empty: `id`, also when `lo >= hi`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag.new((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.fold_between(1, 3) == 1 and b.fold_between(2, 4) == 6`
    pub fn fold_between(self, lo: K, hi: K) -> T = todo

    /// Yield all elements in ascending order. O(n).
    ///
    /// cost: O(1) amortized per step
    /// empty: yields nothing
    /// order: ascending, each key repeated by its count
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag.new((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.each().collect() == [1, 3, 3]`
    pub fn each(self) -> yields K = todo

/// Monoid-augmented ordered unique set maintaining `op` over `leaf(k)` on every subtree (W10 operation values).
/// Covers dynamic range sums and consecutive-gap monoids (`cf2066e`) in O(log n).
///
/// Every fold combines `leaf(k)` over the chosen keys in ascending order, starting from `id`;
/// `op` need not be commutative. An empty selection folds to `id`.
pub struct AugSortedSet<K, T>:
    _len: Int

    /// Create an empty `AugSortedSet<K, T>` with associative `op`, identity `id`, and key map `leaf`. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet.new((x, y) => x + y, 0, k => k); s.all == 0`
    pub fn new(op: (T, T) -> T, id: T, leaf: K -> T) -> AugSortedSet<K, T> = todo

    /// Preallocate treap node pool for at least `cap` keys. O(cap).
    ///
    /// cost: O(cap) when the pool grows, O(1) otherwise
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet.new((x, y) => x + y, 0, k => k); s.reserve(100); s.len == 0`
    pub fn reserve(mut self, cap: Int) = todo

    /// Number of distinct keys. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet.new((x, y) => x + y, 0, k => k); s.add(3); s.add(3); s.len == 1`
    pub @get fn len(self) -> Int = todo

    /// Whether the set is empty. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet.new((x, y) => x + y, 0, k => k); s.is_empty`
    pub @get fn is_empty(self) -> Bool = todo

    /// Fold of all keys in the set. O(1).
    ///
    /// cost: O(1)
    /// empty: `id`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet.new((x, y) => x + y, 0, k => k); s.add(3); s.add(1); s.add(3); s.all == 4`
    pub @get fn all(self) -> T = todo

    /// Insert `x`; returns true if `x` was new. O(log n).
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet.new((x, y) => x + y, 0, k => k); s.add(3) and not s.add(3)`
    pub fn add(mut self, x: K) -> Bool = todo

    /// Remove `x`; returns true if `x` was present. O(log n).
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet.new((x, y) => x + y, 0, k => k); s.add(3); s.remove(3) and not s.remove(3) and s.all == 0`
    pub fn remove(mut self, x: K) -> Bool = todo

    /// Whether `x` is present. O(log n).
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet.new((x, y) => x + y, 0, k => k); s.add(3); s.contains(3) and not s.contains(1)`
    pub fn contains(self, x: K) -> Bool = todo

    /// Number of keys strictly less than `k`. O(log n).
    ///
    /// cost: O(log n) expected
    /// empty: `0`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet.new((x, y) => x + y, 0, k => k); s.add(3); s.add(1); s.rank(3) == 1 and s.rank(4) == 2`
    pub fn rank(self, k: K) -> Int = todo

    /// The `i`-th smallest key (0-based), or `None`. O(log n).
    ///
    /// `None` for any `i` outside `0..len`.
    ///
    /// cost: O(log n) expected
    /// empty: `None`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet.new((x, y) => x + y, 0, k => k); s.add(3); s.add(1); s.nth(1) == Some(3) and s.nth(2) == None`
    pub fn nth(self, i: Int) -> ?K = todo

    /// Smallest key, or `None`. O(1), kept current by every update.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// ties: none: keys are distinct
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet.new((x, y) => x + y, 0, k => k); s.add(3); s.add(1); s.first == Some(1)`
    pub @get fn first(self) -> ?K = todo

    /// Largest key, or `None`. O(1), kept current by every update.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// ties: none: keys are distinct
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet.new((x, y) => x + y, 0, k => k); s.add(3); s.add(1); s.last == Some(3)`
    pub @get fn last(self) -> ?K = todo

    /// Fold keys `< hi`. O(log n).
    ///
    /// cost: O(log n) expected
    /// empty: `id`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet.new((x, y) => x + y, 0, k => k); s.add(1); s.add(3); s.add(5); s.fold_lt(5) == 4`
    pub fn fold_lt(self, hi: K) -> T = todo

    /// Fold keys `<= hi`. O(log n).
    ///
    /// cost: O(log n) expected
    /// empty: `id`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet.new((x, y) => x + y, 0, k => k); s.add(1); s.add(3); s.add(5); s.fold_le(5) == 9`
    pub fn fold_le(self, hi: K) -> T = todo

    /// Fold keys `>= lo`. O(log n).
    ///
    /// cost: O(log n) expected
    /// empty: `id`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet.new((x, y) => x + y, 0, k => k); s.add(1); s.add(3); s.add(5); s.fold_ge(3) == 8`
    pub fn fold_ge(self, lo: K) -> T = todo

    /// Fold keys `> lo`. O(log n).
    ///
    /// cost: O(log n) expected
    /// empty: `id`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet.new((x, y) => x + y, 0, k => k); s.add(1); s.add(3); s.add(5); s.fold_gt(3) == 5`
    pub fn fold_gt(self, lo: K) -> T = todo

    /// Fold keys in `[lo, hi)`. O(log n).
    ///
    /// `lo >= hi` selects nothing and gives `id`.
    ///
    /// cost: O(log n) expected
    /// empty: `id`, also when `lo >= hi`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet.new((x, y) => x + y, 0, k => k); s.add(1); s.add(3); s.add(5); s.fold_between(1, 5) == 4 and s.fold_between(4, 2) == 0`
    pub fn fold_between(self, lo: K, hi: K) -> T = todo

    /// Yield all keys in ascending order. O(n).
    ///
    /// cost: O(1) amortized per step
    /// empty: yields nothing
    /// order: ascending
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet.new((x, y) => x + y, 0, k => k); s.add(3); s.add(1); s.each().collect() == [1, 3]`
    pub fn each(self) -> yields K = todo

/// O(1) point add, O(sqrt(N)) k-th order statistic over an integer value range `r`.
/// Designed as a Mo's algorithm companion for value-range frequency tables.
///
/// `N` is `r.len`; the table keeps one count per value and one per block of about `sqrt(N)`
/// values. Writes outside `r` trap; reads outside `r` see a count of 0.
pub struct BucketCount:
    _lo: Int
    _hi: Int

    /// O(r.len). Create an empty bucketed frequency table over `r`.
    ///
    /// cost: O(r.len)
    /// empty: an empty `r` gives a table every write traps on
    /// traps: never
    /// example: `var b = BucketCount.new(0..10); b.len == 0`
    pub fn new(r: Range<Int>) -> BucketCount = todo

    /// O(1). Add `delta` (default 1; may be negative) copies of `x`.
    ///
    /// A negative `delta` removes copies; the count of `x` may not go below 0.
    ///
    /// cost: O(1)
    /// traps: when `x` is not in `r`; when the count of `x` would become negative
    /// example: `var b = BucketCount.new(0..10); b.add(3); b.add(7, 2); b.count(7) == 2 and b.len == 3`
    pub fn add(mut self, x: Int, delta: Int = 1) = todo

    /// O(1). Remove `delta` (default 1) copies of `x`.
    ///
    /// The same as `add(x, -delta)`.
    ///
    /// cost: O(1)
    /// traps: when `x` is not in `r`; when the count of `x` would become negative
    /// example: `var b = BucketCount.new(0..10); b.add(7, 2); b.remove(7); b.count(7) == 1`
    pub fn remove(mut self, x: Int, delta: Int = 1) = todo

    /// O(1). Multiplicity of `x`.
    ///
    /// cost: O(1)
    /// traps: never; an `x` outside `r` has count 0
    /// example: `var b = BucketCount.new(0..10); b.add(3); b.count(3) == 1 and b.count(4) == 0 and b.count(99) == 0`
    pub fn count(self, x: Int) -> Int = todo

    /// O(sqrt(N)). Number of elements strictly less than `x`.
    ///
    /// Any `x` works: below `r` gives 0, at or above `r.end` gives `len`.
    ///
    /// cost: O(sqrt(N))
    /// empty: `0`
    /// traps: never
    /// example: `var b = BucketCount.new(0..10); b.add(3); b.add(7, 2); b.count_lt(7) == 1 and b.count_lt(8) == 3`
    pub fn count_lt(self, x: Int) -> Int = todo

    /// O(sqrt(N)). The `k`-th smallest element, 0-based like `nth`: `kth(0)` is the smallest.
    ///
    /// Copies count: with elements `3, 7, 7`, `kth(1)` and `kth(2)` are both `7`.
    ///
    /// cost: O(sqrt(N))
    /// ties: none: the `k`-th element is one value
    /// traps: when `k` is not in `0..len`
    /// example: `var b = BucketCount.new(0..10); b.add(3); b.add(7, 2); b.kth(0) == 3 and b.kth(1) == 7 and b.kth(2) == 7`
    pub fn kth(self, k: Int) -> Int = todo

    /// O(1). Total number of elements (with multiplicity).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var b = BucketCount.new(0..10); b.add(4, 3); b.len == 3`
    pub @get fn len(self) -> Int = todo

    /// O(1). Whether the table is empty.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var b = BucketCount.new(0..10); b.add(4); b.remove(4); b.is_empty`
    pub @get fn is_empty(self) -> Bool = todo

    /// O(r.len). Reset all counts to zero.
    ///
    /// cost: O(r.len)
    /// traps: never
    /// example: `var b = BucketCount.new(0..10); b.add(4); b.clear(); b.count(4) == 0 and b.len == 0`
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
