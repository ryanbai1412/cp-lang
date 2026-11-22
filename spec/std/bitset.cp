//! A dense set of positions of a domain, 64 per word.

/// One bit per position of `D`, in `ceil(K / 64)` words.
/// `Bitset<D>` is an explicit type you ask for; `[D]Bool` stays one byte
/// per element and never turns into this behind your back.
/// Single-bit operations are O(1); whole-set operations are O(K / 64).
pub struct Bitset<D>:
    w: List<U64>

    /// The empty set, every word zero; the same set as `empty()`.
    ///
    /// cost: O(K / 64) time and space
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>().len == 0`
    pub fn new() -> Bitset<D> = todo

    /// The empty set, matching `Subset<D>.empty` (a call here: it is O(K / 64)); the same as `new()`.
    ///
    /// cost: O(K / 64) time and space
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.empty().is_empty`
    pub fn empty() -> Bitset<D> = todo

    /// The full set containing every position of `D`, matching `Subset<D>.full`. No bit past the last
    /// position is ever set, so `len` is `D.K`.
    ///
    /// cost: O(K / 64) time and space
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.full().len == 10`
    pub fn full() -> Bitset<D> = todo

    /// The singleton set `{i}`, matching `Subset<D>.single(i)`.
    ///
    /// cost: O(K / 64) time and space
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.single(D(4)).each().collect() == [D(4)]`
    pub fn single(i: D) -> Bitset<D> = todo

    /// The set of positions where `a` is true.
    ///
    /// cost: O(K) time, O(K / 64) space
    /// empty: the empty set when no element is true
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.from([D] v => Int(v) % 4 == 0).each().collect() == [D(0), D(4), D(8)]`
    pub fn from(a: [D]Bool) -> Bitset<D> = todo

    /// Build a bitset from a list or generator of positions `D`. Repeated positions are set once.
    ///
    /// cost: O(K / 64 + xs.len) time, O(K / 64) space
    /// empty: the empty set
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(3), D(1), D(3)]).each().collect() == [D(1), D(3)]`
    pub fn of(xs: List<D>) -> Bitset<D> = todo

    /// Whether position `i` is set; `i in b` calls it.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain D = 0..10; let b = Bitset<D>.single(D(4)); b.contains(D(4)) and not b.contains(D(5))`
    pub fn contains(self, i: D) -> Bool = todo

    /// Set the bit at position `i`; returns `true` if `i` was not already set (matches `Set.add`).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain D = 0..10; var b = Bitset<D>(); let x = b.add(D(2)); let y = b.add(D(2)); x and not y and b.len == 1`
    pub fn add(mut self, i: D) -> Bool = todo

    /// Clear the bit at position `i`; returns `true` if `i` was set (matches `Set.remove`).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain D = 0..10; var b = Bitset<D>.single(D(2)); let x = b.remove(D(2)); let y = b.remove(D(2)); x and not y and b.is_empty`
    pub fn remove(mut self, i: D) -> Bool = todo

    /// Flip the bit at position `i`: set it if clear, clear it if set.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain D = 0..10; var b = Bitset<D>.single(D(2)); b.toggle(D(2)); b.toggle(D(5)); b.each().collect() == [D(5)]`
    pub fn toggle(mut self, i: D) = todo

    /// A copy of `self` with `i` added; `self` is unchanged.
    ///
    /// cost: O(K / 64) time and space
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.single(D(1)).with(D(3)).each().collect() == [D(1), D(3)]`
    pub fn with(self, i: D) -> Bitset<D> = todo

    /// A copy of `self` with `i` removed; `self` is unchanged. Removing a non-member copies.
    ///
    /// cost: O(K / 64) time and space
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.full().without(D(0)).len == 9`
    pub fn without(self, i: D) -> Bitset<D> = todo

    /// A copy of `self` with `i` added if `on` or removed otherwise, matching `Subset<D>.set`.
    ///
    /// cost: O(K / 64) time and space
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.single(D(1)).set(D(1), false).is_empty`
    /// example: `domain D = 0..10; Bitset<D>().set(D(7), true).each().collect() == [D(7)]`
    pub fn set(self, i: D, on: Bool) -> Bitset<D> = todo

    /// The complement of `self` within `D`, matching `Subset<D>.complement`; bits past the last position
    /// stay clear.
    ///
    /// cost: O(K / 64) time and space
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).complement().len == 7`
    pub fn complement(self) -> Bitset<D> = todo

    /// Remove every position from the set.
    ///
    /// cost: O(K / 64)
    /// traps: never
    /// example: `domain D = 0..10; var b = Bitset<D>.full(); b.clear(); b.is_empty`
    pub fn clear(mut self) = todo

    /// True when some position in this set satisfies `pred`. Visits members in increasing order and stops
    /// at the first that satisfies it.
    ///
    /// cost: O(K / 64 + visited) words scanned and calls of `pred`
    /// empty: `false`
    /// traps: never (beyond what `pred` traps on)
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).any(d => Int(d) > 5)`
    /// example: `domain D = 0..10; not Bitset<D>().any(d => true)`
    pub fn any(self, pred: D -> Bool) -> Bool = todo
    /// The number of positions in the set; the count is kept current by every update.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).len == 3`
    pub @get fn len(self) -> Int = todo

    /// The same number as `len`, under the name C++ gives it.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).count == 3`
    pub @get fn count(self) -> Int = todo

    /// Whether the set has no members. O(1): the count is kept current by every update.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>().is_empty and not Bitset<D>.full().is_empty`
    pub @get fn is_empty(self) -> Bool = todo

    /// The smallest position in the set, or `None`.
    ///
    /// cost: O(K / 64)
    /// empty: `None`
    /// ties: none possible; positions are distinct
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).first() == Some(D(1))`
    pub fn first(self) -> ?D = todo

    /// The smallest position in the set satisfying `pred`, or `None`, as `seq.find`. Members are
    /// visited in increasing order, so `pred` is called only up to the answer.
    ///
    /// cost: O(K / 64 + visited)
    /// empty: `None`
    /// ties: none possible; the smallest qualifying position wins
    /// traps: never (beyond what `pred` traps on)
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).find(d => Int(d) > 2) == Some(D(3))`
    pub fn find(self, pred: D -> Bool) -> ?D = todo

    /// The largest position in the set satisfying `pred` (any, by default), or `None`. Members are
    /// visited in decreasing order.
    ///
    /// cost: O(K / 64 + visited)
    /// empty: `None`
    /// ties: none possible; the largest qualifying position wins
    /// traps: never (beyond what `pred` traps on)
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).last() == Some(D(8))`
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).last(d => Int(d) < 8) == Some(D(3))`
    pub fn last(self, pred: D -> Bool = d => true) -> ?D = todo

    /// Replace `self` with `self | o` in place, with no allocation.
    ///
    /// cost: O(K / 64)
    /// traps: never
    /// example: `domain D = 0..10; var b = Bitset<D>.single(D(1)); b.union(Bitset<D>.single(D(2))); b.len == 2`
    pub fn union(mut self, o: Bitset<D>) = todo

    /// Replace `self` with `self & o` in place, with no allocation.
    ///
    /// cost: O(K / 64)
    /// traps: never
    /// example: `domain D = 0..10; var b = Bitset<D>.of([D(1), D(2)]); b.intersect(Bitset<D>.single(D(2))); b.each().collect() == [D(2)]`
    pub fn intersect(mut self, o: Bitset<D>) = todo

    /// Clear every bit set in `o` from `self` in place: `self - o`.
    ///
    /// cost: O(K / 64)
    /// traps: never
    /// example: `domain D = 0..10; var b = Bitset<D>.of([D(1), D(2)]); b.subtract(Bitset<D>.single(D(2))); b.each().collect() == [D(1)]`
    pub fn subtract(mut self, o: Bitset<D>) = todo

    /// Replace `self` with `self ^ o` in place.
    ///
    /// cost: O(K / 64)
    /// traps: never
    /// example: `domain D = 0..10; var b = Bitset<D>.of([D(1), D(2)]); b.symmetric(Bitset<D>.of([D(2), D(3)])); b.each().collect() == [D(1), D(3)]`
    pub fn symmetric(mut self, o: Bitset<D>) = todo

    /// `a | b`: a new set of the positions in either set.
    ///
    /// cost: O(K / 64) time and space
    /// traps: never
    /// example: `domain D = 0..10; (Bitset<D>.single(D(1)) | Bitset<D>.single(D(2))).len == 2`
    pub fn bitor(self, o: Bitset<D>) -> Bitset<D> = todo

    /// `a & b`: a new set of the positions in both sets.
    ///
    /// cost: O(K / 64) time and space
    /// traps: never
    /// example: `domain D = 0..10; (Bitset<D>.of([D(1), D(2)]) & Bitset<D>.of([D(2), D(3)])).each().collect() == [D(2)]`
    pub fn bitand(self, o: Bitset<D>) -> Bitset<D> = todo

    /// `a ^ b`: a new set of the positions in exactly one of the sets.
    ///
    /// cost: O(K / 64) time and space
    /// traps: never
    /// example: `domain D = 0..10; (Bitset<D>.of([D(1), D(2)]) ^ Bitset<D>.of([D(2), D(3)])).each().collect() == [D(1), D(3)]`
    pub fn bitxor(self, o: Bitset<D>) -> Bitset<D> = todo

    /// Do the two sets share a position? Stops at the first shared word; `(a & b).len > 0` without
    /// building `a & b`.
    ///
    /// cost: O(K / 64), less on an early hit
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(2)]).meets(Bitset<D>.of([D(2), D(3)]))`
    /// example: `domain D = 0..10; not Bitset<D>.single(D(1)).meets(Bitset<D>.single(D(2)))`
    pub fn meets(self, o: Bitset<D>) -> Bool = todo

    /// A new set with every position `k` places up (down for a negative `k`); positions that leave `D`
    /// are dropped. Position `v` moves to the position whose ordinal is `v`'s plus `k`.
    ///
    /// cost: O(K / 64) time and space, for any `k`
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).shifted(2).each().collect() == [D(3), D(5)]`
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).shifted(-2).each().collect() == [D(1), D(6)]`
    pub fn shifted(self, k: Int) -> Bitset<D> = todo

    /// Shift every position `k >= 0` places up in place; bits past `D` drop.
    ///
    /// cost: O(K / 64)
    /// traps: when `k < 0`
    /// example: `domain D = 0..10; var b = Bitset<D>.of([D(1), D(3), D(8)]); b.shift_up(1); b.each().collect() == [D(2), D(4), D(9)]`
    pub fn shift_up(mut self, k: Int) = todo

    /// In-place `self |= self << k` for `k >= 0`, with zero allocation: the knapsack step "every
    /// reachable sum, and every reachable sum plus `k`". Bits past `D` drop.
    ///
    /// cost: O(K / 64)
    /// traps: when `k < 0`
    /// example: `domain D = 0..10; var b = Bitset<D>.of([D(1), D(3), D(8)]); b.shift_or(2); b.each().collect() == [D(1), D(3), D(5), D(8)]`
    pub fn shift_or(mut self, k: Int) = todo

    /// The positions in the set, in increasing order; `for v in b` walks the same sequence.
    ///
    /// cost: O(K / 64 + len) to walk, O(1) per member amortized
    /// empty: yields nothing
    /// order: increasing
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(8), D(1)]).each().collect() == [D(1), D(8)]`
    pub fn each(self) -> yields D = todo

    /// The first position in the set at or after `i`, or `None`.
    ///
    /// cost: O(K / 64)
    /// empty: `None`
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).next(D(3)) == Some(D(3))`
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).next(D(4)) == Some(D(8))`
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).next(D(9)) == None`
    pub fn next(self, i: D) -> ?D = todo

    /// The last position in the set at or before `i`, or `None` (the mirror of `next`).
    ///
    /// cost: O(K / 64)
    /// empty: `None`
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).prev(D(3)) == Some(D(3))`
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).prev(D(2)) == Some(D(1))`
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).prev(D(0)) == None`
    pub fn prev(self, i: D) -> ?D = todo

    /// The number of positions in the set strictly before `i`.
    ///
    /// cost: O(K / 64)
    /// empty: zero
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).count_below(D(8)) == 2`
    pub fn count_below(self, i: D) -> Int = todo

    /// All subset sums in `0..D.K` reachable from non-negative `weights`: the position with ordinal `s`
    /// is set exactly when some sub-multiset of `weights` sums to `s`. The empty sub-multiset makes `0`
    /// reachable; sums of `D.K` or more are dropped.
    /// Groups duplicate weights with binary carries so the work is O(K sqrt(S) / 64).
    ///
    /// cost: O(K sqrt(S) / 64) time, where S is the sum of `weights`; O(K / 64) space
    /// empty: `{D.first}` (just the sum 0), or the empty set when `D` is empty
    /// traps: when a weight is negative
    /// example: `domain S = 0..=10; Bitset<S>.subset_sums([3, 5, 5]).each().map(s => Int(s)).collect() == [0, 3, 5, 8, 10]`
    pub fn subset_sums(weights: List<Int>) -> Bitset<D> = todo

    /// Shift all bits down by `k >= 0` positions in place (`i -> i - k`); bits below `D` drop.
    ///
    /// cost: O(K / 64)
    /// traps: when `k < 0`
    /// example: `domain D = 0..10; var b = Bitset<D>.of([D(1), D(3), D(8)]); b.shift_down(1); b.each().collect() == [D(0), D(2), D(7)]`
    pub fn shift_down(mut self, k: Int) = todo

    /// Count set bits in `r` (accepts cut ranges via decision 124).
    ///
    /// cost: O(r.len / 64 + 1)
    /// empty: zero
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).count_range(1..8) == 2`
    pub fn count_range(self, r: Range<D>) -> Int = todo

    /// Set (or, with `on: false`, clear) every bit in `r` in place.
    ///
    /// cost: O(r.len / 64 + 1)
    /// empty: no effect
    /// traps: never
    /// example: `domain D = 0..10; var b = Bitset<D>.of([D(1), D(3), D(8)]); b.set_range(2..5); b.len == 5`
    /// example: `domain D = 0..10; var b = Bitset<D>.of([D(1), D(3), D(8)]); b.set_range(0..4, on: false); b.each().collect() == [D(8)]`
    pub fn set_range(mut self, r: Range<D>, on: Bool = true) = todo

    /// Flip every bit in `r` in place.
    ///
    /// cost: O(r.len / 64 + 1)
    /// empty: no effect
    /// traps: never
    /// example: `domain D = 0..10; var b = Bitset<D>.of([D(1), D(3), D(8)]); b.flip_range(0..4); b.each().collect() == [D(0), D(2), D(8)]`
    pub fn flip_range(mut self, r: Range<D>) = todo

    /// The `k`-th set position (0-based, in increasing order), or `None` if `k >= len`.
    ///
    /// cost: O(K / 64)
    /// empty: `None`
    /// traps: when `k < 0`
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).nth(2) == Some(D(8))`
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).nth(3) == None`
    pub fn nth(self, k: Int) -> ?D = todo

    /// The set positions in descending order.
    ///
    /// cost: O(K / 64 + len) to walk
    /// empty: yields nothing
    /// order: decreasing
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).each_rev().collect() == [D(8), D(3), D(1)]`
    pub fn each_rev(self) -> yields D = todo

    fn trim(mut self) = todo

/// Runtime-sized 64-bit word bitset over `0..cap` when the width is computed dynamically without a domain.
/// Single-bit ops are O(1); word-parallel ops are O(cap / 64).
pub struct DynBitset:
    _w: List<U64>
    _cap: Int

    /// Create an all-zero `DynBitset` of width `cap`: bits `0..cap`.
    ///
    /// cost: O(cap / 64) time and space
    /// traps: when `cap < 0`
    /// example: `DynBitset(10).len == 0`
    pub fn new(cap: Int) -> DynBitset = todo

    /// Create an all-one `DynBitset` of width `cap`; bits at or above `cap` stay clear.
    ///
    /// cost: O(cap / 64) time and space
    /// traps: when `cap < 0`
    /// example: `DynBitset.full(10).len == 10`
    pub fn full(cap: Int) -> DynBitset = todo

    /// The bit capacity: valid indices are `0..cap`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `DynBitset(10).cap == 10`
    pub @get fn cap(self) -> Int = todo

    /// The number of set bits, kept current by every update.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var d = DynBitset(10); d.add(1); d.add(3); d.len == 2`
    pub @get fn len(self) -> Int = todo

    /// Whether no bits are set, kept current by every update.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `DynBitset(10).is_empty and not DynBitset.full(1).is_empty`
    pub @get fn is_empty(self) -> Bool = todo

    /// Whether bit `i` is set.
    ///
    /// cost: O(1)
    /// traps: when `i` is not in `0..cap`
    /// example: `var d = DynBitset(10); d.add(3); d.contains(3) and not d.contains(4)`
    pub fn contains(self, i: Int) -> Bool = todo

    /// Set bit `i` to `on` in place (W8 `IndexSet` backing `b[i] = on`).
    ///
    /// cost: O(1)
    /// traps: when `i` is not in `0..cap`
    /// example: `var d = DynBitset(10); d.set(4); d.set(5, true); d.set(4, false); d.each().collect() == [5]`
    pub fn set(mut self, i: Int, on: Bool = true) = todo

    /// Set bit `i` to 1; returns `true` if bit `i` was previously 0 (matches `Bitset.add`).
    ///
    /// cost: O(1)
    /// traps: when `i` is not in `0..cap`
    /// example: `var d = DynBitset(10); let x = d.add(2); let y = d.add(2); x and not y`
    pub fn add(mut self, i: Int) -> Bool = todo

    /// Clear bit `i` to 0; returns `true` if bit `i` was previously 1 (matches `Bitset.remove`).
    ///
    /// cost: O(1)
    /// traps: when `i` is not in `0..cap`
    /// example: `var d = DynBitset(10); d.add(2); let x = d.remove(2); let y = d.remove(2); x and not y and d.is_empty`
    pub fn remove(mut self, i: Int) -> Bool = todo

    /// Flip bit `i`.
    ///
    /// cost: O(1)
    /// traps: when `i` is not in `0..cap`
    /// example: `var d = DynBitset(10); d.toggle(6); d.toggle(6); d.toggle(7); d.each().collect() == [7]`
    pub fn toggle(mut self, i: Int) = todo

    /// Clear all bits; the capacity is unchanged.
    ///
    /// cost: O(cap / 64)
    /// traps: never
    /// example: `var d = DynBitset.full(10); d.clear(); d.is_empty and d.cap == 10`
    pub fn clear(mut self) = todo

    /// Resize to `new_cap` bits, zero-filling any new high bits. Shrinking drops the bits at or above
    /// `new_cap` (and `len` drops with them).
    ///
    /// cost: O(new_cap / 64) time and space
    /// traps: when `new_cap < 0`
    /// example: `var d = DynBitset(10); d.add(1); d.add(8); d.resize(4); d.len == 1 and d.cap == 4`
    /// example: `var d = DynBitset.full(2); d.resize(5); d.each().collect() == [0, 1]`
    pub fn resize(mut self, new_cap: Int) = todo

    /// The smallest set bit index, or `None` if empty.
    ///
    /// cost: O(cap / 64)
    /// empty: `None`
    /// ties: none possible; the smallest index wins
    /// traps: never
    /// example: `var d = DynBitset(10); d.add(8); d.add(3); d.first() == Some(3)`
    /// example: `DynBitset(10).first() == None`
    pub fn first(self) -> ?Int = todo

    /// The largest set bit index, or `None` if empty.
    ///
    /// cost: O(cap / 64)
    /// empty: `None`
    /// ties: none possible; the largest index wins
    /// traps: never
    /// example: `var d = DynBitset(10); d.add(8); d.add(3); d.last() == Some(8)`
    pub fn last(self) -> ?Int = todo

    /// The smallest set bit `>= i`, or `None`.
    ///
    /// cost: O(cap / 64)
    /// empty: `None`
    /// traps: when `i` is not in `0..=cap`
    /// example: `var d = DynBitset(10); d.add(3); d.add(8); d.next(3) == Some(3) and d.next(4) == Some(8) and d.next(9) == None`
    pub fn next(self, i: Int) -> ?Int = todo

    /// The largest set bit `<= i`, or `None`; `i` itself counts, as in `Bitset.prev`.
    ///
    /// cost: O(cap / 64)
    /// empty: `None`
    /// traps: when `i` is not in `0..cap`
    /// example: `var d = DynBitset(10); d.add(1); d.add(3); d.prev(3) == Some(3) and d.prev(2) == Some(1) and d.prev(0) == None`
    pub fn prev(self, i: Int) -> ?Int = todo

    /// The number of set bits strictly below `i`.
    ///
    /// cost: O(cap / 64)
    /// empty: zero
    /// traps: when `i` is not in `0..=cap`
    /// example: `var d = DynBitset(10); d.add(1); d.add(3); d.add(8); d.count_below(8) == 2`
    pub fn count_below(self, i: Int) -> Int = todo

    /// The number of set bits in `r`.
    ///
    /// cost: O(r.len / 64 + 1)
    /// empty: zero
    /// traps: when `r` is non-empty and reaches outside `0..cap`
    /// example: `var d = DynBitset(10); d.add(1); d.add(3); d.add(8); d.count_range(1..8) == 2`
    pub fn count_range(self, r: Range<Int>) -> Int = todo

    /// The `k`-th set bit index (0-based, in increasing order), or `None` if `k >= len`.
    ///
    /// cost: O(cap / 64)
    /// empty: `None`
    /// traps: when `k < 0`
    /// example: `var d = DynBitset(10); d.add(1); d.add(3); d.add(8); d.nth(1) == Some(3) and d.nth(3) == None`
    pub fn nth(self, k: Int) -> ?Int = todo

    /// In-place bitwise OR (`self |= other`).
    ///
    /// cost: O(cap / 64)
    /// traps: when `other.cap != self.cap`
    /// example: `var a = DynBitset(4); a.add(0); var b = DynBitset(4); b.add(2); a.union(b); a.each().collect() == [0, 2]`
    pub fn union(mut self, other: DynBitset) = todo

    /// In-place bitwise AND (`self &= other`).
    ///
    /// cost: O(cap / 64)
    /// traps: when `other.cap != self.cap`
    /// example: `var a = DynBitset.full(4); var b = DynBitset(4); b.add(2); a.intersect(b); a.each().collect() == [2]`
    pub fn intersect(mut self, other: DynBitset) = todo

    /// In-place set difference (`self &= ~other`).
    ///
    /// cost: O(cap / 64)
    /// traps: when `other.cap != self.cap`
    /// example: `var a = DynBitset.full(4); var b = DynBitset(4); b.add(2); a.subtract(b); a.each().collect() == [0, 1, 3]`
    pub fn subtract(mut self, other: DynBitset) = todo

    /// In-place bitwise XOR (`self ^= other`).
    ///
    /// cost: O(cap / 64)
    /// traps: when `other.cap != self.cap`
    /// example: `var a = DynBitset(4); a.add(1); a.add(2); var b = DynBitset(4); b.add(2); b.add(3); a.symmetric(b); a.each().collect() == [1, 3]`
    pub fn symmetric(mut self, other: DynBitset) = todo

    /// Whether `self` and `other` share a set bit; stops at the first shared word.
    ///
    /// cost: O(cap / 64), less on an early hit
    /// traps: when `other.cap != self.cap`
    /// example: `var a = DynBitset(4); a.add(1); var b = DynBitset.full(4); a.meets(b) and not a.meets(DynBitset(4))`
    pub fn meets(self, other: DynBitset) -> Bool = todo

    /// Shift all bits up by `k >= 0` in place (`i -> i + k`); bits reaching `cap` or beyond drop.
    ///
    /// cost: O(cap / 64)
    /// traps: when `k < 0`
    /// example: `var d = DynBitset(10); d.add(1); d.add(3); d.add(8); d.shift_up(2); d.each().collect() == [3, 5]`
    pub fn shift_up(mut self, k: Int) = todo

    /// Shift all bits down by `k >= 0` in place (`i -> i - k`); bits falling below 0 drop.
    ///
    /// cost: O(cap / 64)
    /// traps: when `k < 0`
    /// example: `var d = DynBitset(10); d.add(1); d.add(3); d.add(8); d.shift_down(2); d.each().collect() == [1, 6]`
    pub fn shift_down(mut self, k: Int) = todo

    /// In-place `self |= (self << k)` for `k >= 0`, with zero allocation; bits reaching `cap` drop.
    ///
    /// cost: O(cap / 64)
    /// traps: when `k < 0`
    /// example: `var d = DynBitset(10); d.add(1); d.add(3); d.add(8); d.shift_or(2); d.each().collect() == [1, 3, 5, 8]`
    pub fn shift_or(mut self, k: Int) = todo

    /// The set bit indices in ascending order; `for i in d` walks the same sequence.
    ///
    /// cost: O(cap / 64 + len) to walk
    /// empty: yields nothing
    /// order: increasing
    /// traps: never
    /// example: `var d = DynBitset(10); d.add(8); d.add(1); d.each().collect() == [1, 8]`
    pub fn each(self) -> yields Int = todo

    /// The set bit indices in descending order.
    ///
    /// cost: O(cap / 64 + len) to walk
    /// empty: yields nothing
    /// order: decreasing
    /// traps: never
    /// example: `var d = DynBitset(10); d.add(8); d.add(1); d.each_rev().collect() == [8, 1]`
    pub fn each_rev(self) -> yields Int = todo

    /// All subset sums in `0..cap` reachable from `weights`: bit `s` is set exactly when some
    /// sub-multiset of `weights` sums to `s`. The empty sub-multiset makes `0` reachable (when
    /// `cap > 0`); sums of `cap` or more drop. Duplicate weights are grouped with binary carries, as in
    /// `Bitset.subset_sums`.
    ///
    /// cost: O(cap sqrt(S) / 64) time, where S is the sum of `weights`; O(cap / 64) space
    /// empty: just bit 0 (none when `cap == 0`)
    /// traps: when `cap < 0` or a weight is negative
    /// example: `DynBitset.subset_sums(11, [3, 5, 5]).each().collect() == [0, 3, 5, 8, 10]`
    pub fn subset_sums(cap: Int, weights: List<Int>) -> DynBitset = todo

/// O(bits) per op. Dynamic binary bitwise XOR trie multiset over non-negative `bits`-bit integers (default 60 bits).
pub struct XorTrie:
    _bits: Int

    /// Create an empty `XorTrie` over `bits`-bit non-negative integers (default 60): values in
    /// `0..2^bits`.
    ///
    /// cost: O(1)
    /// traps: when `bits` is not in `1..=62`
    /// example: `XorTrie(3).is_empty`
    pub fn new(bits: Int = 60) -> XorTrie = todo

    /// Reserve pool capacity for at least `nodes` trie nodes, so later adds do not reallocate; the
    /// contents are unchanged.
    ///
    /// cost: O(nodes) time and space
    /// traps: when `nodes < 0`
    /// example: `var t = XorTrie(3); t.reserve(100); t.len == 0`
    pub fn reserve(mut self, nodes: Int) = todo

    /// The total number of stored integers, with multiplicity.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var t = XorTrie(3); t.add(5); t.add(5); t.len == 2`
    pub @get fn len(self) -> Int = todo

    /// True when the trie holds no integers.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var t = XorTrie(3); t.add(5); t.remove(5); t.is_empty`
    pub @get fn is_empty(self) -> Bool = todo

    /// Insert one copy of `x`. `x` is stored as given, after any earlier `xor_all`: the mask applies
    /// only to elements already present.
    ///
    /// cost: O(bits) time, up to `bits` new nodes
    /// traps: when `x` is not in `0..2^bits`
    /// example: `var t = XorTrie(3); t.add(5); t.add(5); t.count(5) == 2`
    pub fn add(mut self, x: Int) = todo

    /// Remove one copy of `x`; returns `false` if absent, leaving the trie unchanged.
    ///
    /// cost: O(bits)
    /// traps: never
    /// example: `var t = XorTrie(3); t.add(5); let x = t.remove(5); let y = t.remove(5); x and not y`
    pub fn remove(mut self, x: Int) -> Bool = todo

    /// Whether `x` is present at least once.
    ///
    /// cost: O(bits)
    /// traps: never
    /// example: `var t = XorTrie(3); t.add(5); t.contains(5) and not t.contains(4)`
    pub fn contains(self, x: Int) -> Bool = todo

    /// The multiplicity of `x` currently stored; zero when absent.
    ///
    /// cost: O(bits)
    /// traps: never
    /// example: `var t = XorTrie(3); t.add(5); t.add(5); t.add(1); t.count(5) == 2 and t.count(2) == 0`
    pub fn count(self, x: Int) -> Int = todo

    /// Apply a lazy global XOR mask: every stored `y` becomes `y ^ mask`. Masks compose (`xor_all(a)`
    /// then `xor_all(b)` is `xor_all(a ^ b)`).
    ///
    /// cost: O(1)
    /// traps: when `mask` is not in `0..2^bits`
    /// example: `var t = XorTrie(3); t.add(1); t.add(5); t.xor_all(1); t.contains(0) and t.contains(4) and not t.contains(1)`
    pub fn xor_all(mut self, mask: Int) = todo

    /// The maximum value of `x ^ y` over stored `y`, or `None` if empty.
    ///
    /// cost: O(bits)
    /// empty: `None`
    /// ties: none possible; the value is unique even when several `y` reach it
    /// traps: when `x` is not in `0..2^bits`
    /// example: `var t = XorTrie(3); t.add(1); t.add(5); t.add(6); t.max_xor(2) == Some(7)`
    /// example: `XorTrie(3).max_xor(2) == None`
    pub fn max_xor(self, x: Int) -> ?Int = todo

    /// The minimum value of `x ^ y` over stored `y`, or `None` if empty.
    ///
    /// cost: O(bits)
    /// empty: `None`
    /// ties: none possible; the value is unique even when several `y` reach it
    /// traps: when `x` is not in `0..2^bits`
    /// example: `var t = XorTrie(3); t.add(1); t.add(5); t.add(6); t.min_xor(2) == Some(3)`
    pub fn min_xor(self, x: Int) -> ?Int = todo

    /// The number of stored `y` (with multiplicity) with `(x ^ y) < limit`. A `limit <= 0` gives 0 and
    /// a `limit >= 2^bits` gives `len`.
    ///
    /// cost: O(bits)
    /// traps: when `x` is not in `0..2^bits`
    /// example: `var t = XorTrie(3); t.add(1); t.add(5); t.add(6); t.count_xor_less(2, 5) == 2`
    pub fn count_xor_less(self, x: Int, limit: Int) -> Int = todo

    /// The `k`-th smallest value of `x ^ y` over stored `y` (0-based, with multiplicity), or `None` if
    /// `k >= len`.
    ///
    /// cost: O(bits)
    /// empty: `None`
    /// ties: equal values `x ^ y` (copies of one `y`) each take their own rank
    /// traps: when `k < 0` or `x` is not in `0..2^bits`
    /// example: `var t = XorTrie(3); t.add(1); t.add(5); t.add(6); t.kth_xor(2, 1) == Some(4)`
    /// example: `var t = XorTrie(3); t.add(1); t.add(5); t.add(6); t.kth_xor(2, 3) == None`
    pub fn kth_xor(self, x: Int, k: Int) -> ?Int = todo

    /// The smallest non-negative integer not present under XOR mask `x` (mex of `{y ^ x}`); `2^bits`
    /// when every value is present.
    ///
    /// cost: O(bits)
    /// empty: zero
    /// traps: when `x` is not in `0..2^bits`
    /// example: `var t = XorTrie(3); t.add(1); t.add(5); t.add(6); t.mex_xor() == 0 and t.mex_xor(1) == 1`
    pub fn mex_xor(self, x: Int = 0) -> Int = todo

    /// Remove all elements and the global mask, keeping `bits` and the reserved pool.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var t = XorTrie(3); t.add(5); t.clear(); t.is_empty and not t.contains(5)`
    pub fn clear(mut self) = todo

/// `a - b`: the positions in `a` that are not in `b`.
impl Sub for Bitset<D>:
    fn sub(a, b) = todo

/// Index of the highest set bit in non-zero `x` (0..64).
fn msb(x: U64) -> Int = todo

impl Show for Bitset<D>:
    fn show(a) = todo
/// The word with only bit `k` set.
fn bit(k: Int) -> U64 = todo

/// The word with its low `n` bits set, for `n` in 0..=64.
fn low(n: Int) -> U64 = todo

@test
fn single_bits_across_word_borders() = todo

@test
fn set_algebra_matches_bool_arrays() = todo

@test
fn shifted_matches_moving_each_position() = todo

@test
fn subset_sum_by_shifting() = todo

@test
fn membership_and_a_signed_shift() = todo

@test
fn full_contains_every_position_and_nothing_past_end() = todo

@test
fn nav_helpers_and_in_place_shifts() = todo

@test
fn subset_sums_matches_naive_with_duplicates() = todo

@test
fn subset_parity_constructors_and_complement() = todo
