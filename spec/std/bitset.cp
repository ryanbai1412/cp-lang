//! A dense set of positions of a domain, 64 per word.

/// One bit per position of `D`, in `ceil(K / 64)` words.
/// `Bitset<D>` is an explicit type you ask for; `[D]Bool` stays one byte
/// per element and never turns into this behind your back.
/// Single-bit operations are O(1); whole-set operations are O(K / 64).
pub struct Bitset<D>:
    w: List<U64>

    /// The empty set. O(K / 64).
    pub fn new() -> Bitset<D> = todo

    /// The empty set, matching `Subset<D>.empty`. O(K / 64).
    pub @get fn empty() -> Bitset<D> = todo

    /// The full set containing every position of `D`, matching `Subset<D>.full`. O(K / 64).
    pub @get fn full() -> Bitset<D> = todo

    /// The singleton set `{i}`, matching `Subset<D>.single(i)`. O(K / 64).
    pub fn single(i: D) -> Bitset<D> = todo

    /// The set of positions where `a` is true. O(D.K).
    pub fn from(a: [D]Bool) -> Bitset<D> = todo

    /// Build a bitset from a list or generator of positions `D`. O(D.K / 64 + xs.len).
    pub fn of(xs: List<D>) -> Bitset<D> = todo

    /// Whether position `i` is set. O(1).
    pub fn contains(self, i: D) -> Bool = todo

    /// Set the bit at position `i`; returns `true` if `i` was not already set (matches `Set.add`). O(1).
    pub fn add(mut self, i: D) -> Bool = todo

    /// Clear the bit at position `i`; returns `true` if `i` was set (matches `Set.remove`). O(1).
    pub fn remove(mut self, i: D) -> Bool = todo

    /// Flip the bit at position `i`. O(1).
    pub fn toggle(mut self, i: D) = todo

    /// Copy of `self` with `i` added. O(K / 64).
    pub fn with(self, i: D) -> Bitset<D> = todo

    /// Copy of `self` with `i` removed. O(K / 64).
    pub fn without(self, i: D) -> Bitset<D> = todo

    /// Copy of `self` with `i` added if `on` or removed otherwise, matching `Subset<D>.set`. O(K / 64).
    pub fn set(self, i: D, on: Bool) -> Bitset<D> = todo

    /// Complement of `self` within `D`, matching `Subset<D>.complement`. O(K / 64).
    pub @get fn complement(self) -> Bitset<D> = todo

    /// Remove every position from the set. O(K / 64).
    pub fn clear(mut self) = todo

    /// [rename] First position in this set satisfying `pred`, or `none` (replaces `find`). O(K / 64 + visited).
    pub fn first_where(self, pred: D -> Bool) -> ?D = todo

    /// True when some position in this set satisfies `pred`. O(K / 64 + visited).
    pub fn any(self, pred: D -> Bool) -> Bool = todo
    /// The number of positions in the set.
    pub @get fn len(self) -> Int = todo

    /// The same number as `len`, under the name C++ gives it.
    pub @get fn count(self) -> Int = todo

    /// Whether the set has no members; stops at the first non-zero word. O(K / 64).
    pub @get fn is_empty(self) -> Bool = todo

    /// The smallest position in the set, or `none`. O(K / 64).
    pub @get fn first(self) -> ?D = todo

    /// The largest position in the set, or `none`. O(K / 64).
    pub @get fn last(self) -> ?D = todo

    /// Replace `self` with `self | o` in place. O(|D| / 64).
    pub fn union(mut self, o: Bitset<D>) = todo

    /// Replace `self` with `self & o` in place. O(|D| / 64).
    pub fn intersect(mut self, o: Bitset<D>) = todo

    /// Clear every bit set in `o` from `self` in place. O(|D| / 64).
    pub fn subtract(mut self, o: Bitset<D>) = todo

    /// Replace `self` with `self ^ o` in place. O(|D| / 64).
    pub fn symmetric(mut self, o: Bitset<D>) = todo

    /// `a | b`: the positions in either set.
    pub fn bitor(self, o: Bitset<D>) -> Bitset<D> = todo

    /// `a & b`: the positions in both sets.
    pub fn bitand(self, o: Bitset<D>) -> Bitset<D> = todo

    /// `a ^ b`: the positions in exactly one of the sets.
    pub fn bitxor(self, o: Bitset<D>) -> Bitset<D> = todo

    /// Do the two sets share a position? Stops at the first shared word.
    pub fn meets(self, o: Bitset<D>) -> Bool = todo

    /// Move every position `k` places up (down for a negative `k`); positions that leave `D` are dropped.
    /// O(K / 64) for both positive and negative `k`.
    pub fn shifted(self, k: Int) -> Bitset<D> = todo

    /// Shift every position `k >= 0` places up in place; bits past `D` drop. O(K / 64).
    pub fn shift_up(mut self, k: Int) = todo

    /// In-place `self |= self << k` for `k >= 0`, with zero allocation. O(K / 64).
    pub fn shift_or(mut self, k: Int) = todo

    /// The positions in the set, in increasing order.
    pub fn each(self) -> yields D = todo

    /// The first position in the set at or after `i`, or `none`. O(K / 64).
    pub fn next(self, i: D) -> ?D = todo

    /// The last position in the set strictly before `i`, or `none`. O(K / 64).
    pub fn prev(self, i: D) -> ?D = todo

    /// Number of positions in the set strictly before `i`. O(K / 64).
    pub fn count_below(self, i: D) -> Int = todo

    /// All subset sums in `0..D.K` reachable from non-negative `weights`.
    /// Groups duplicate weights with binary carries so the work is O(K sqrt(S) / 64).
    pub fn subset_sums(weights: List<Int>) -> Bitset<D> = todo

    /// Shift all bits down by `k` positions in place (`i -> i - k`). O(K / 64).
    pub fn shift_down(mut self, k: Int) = todo

    /// Count set bits in `r` (accepts cut ranges via decision 124). O(r.len / 64).
    pub fn count_range(self, r: Range<D>) -> Int = todo

    /// Set or clear every bit in `r` in place. O(r.len / 64).
    pub fn set_range(mut self, r: Range<D>, on: Bool = true) = todo

    /// Flip every bit in `r` in place. O(r.len / 64).
    pub fn flip_range(mut self, r: Range<D>) = todo

    /// The `k`-th set position (0-based), or `none` if `k >= len`. O(K / 64).
    pub fn nth(self, k: Int) -> ?D = todo

    /// Yield set positions in descending order. O(K / 64 + len).
    pub fn each_rev(self) -> yields D = todo

    fn trim(mut self) = todo

/// Runtime-sized 64-bit word bitset over `0..cap` when the width is computed dynamically without a domain.
/// Single-bit ops are O(1); word-parallel ops are O(cap / 64).
pub struct DynBitset:
    _w: List<U64>
    _cap: Int

    /// Create an all-zero `DynBitset` of width `cap`. O(cap / 64).
    pub fn new(cap: Int) -> DynBitset = todo

    /// Create an all-one `DynBitset` of width `cap`. O(cap / 64).
    pub fn full(cap: Int) -> DynBitset = todo

    /// Bit capacity of the bitset. O(1).
    pub @get fn cap(self) -> Int = todo

    /// Number of set bits. O(cap / 64).
    pub @get fn len(self) -> Int = todo

    /// Whether no bits are set. O(cap / 64).
    pub @get fn is_empty(self) -> Bool = todo

    /// Whether bit `i` is set. O(1).
    pub fn contains(self, i: Int) -> Bool = todo

    /// Set bit `i` to `on` in place (W8 `IndexSet` backing `b[i] = on`). O(1).
    pub fn set(mut self, i: Int, on: Bool = true) = todo

    /// Set bit `i` to 1; returns `true` if bit `i` was previously 0 (matches `Bitset.add`). O(1).
    pub fn add(mut self, i: Int) -> Bool = todo

    /// Clear bit `i` to 0; returns `true` if bit `i` was previously 1 (matches `Bitset.remove`). O(1).
    pub fn remove(mut self, i: Int) -> Bool = todo

    /// Flip bit `i`. O(1).
    pub fn toggle(mut self, i: Int) = todo

    /// Clear all bits. O(cap / 64).
    pub fn clear(mut self) = todo

    /// Resize to `new_cap` bits, zero-filling any new high bits. O(new_cap / 64).
    pub fn resize(mut self, new_cap: Int) = todo

    /// Smallest set bit index, or `none` if empty. O(cap / 64).
    pub @get fn first(self) -> ?Int = todo

    /// Largest set bit index, or `none` if empty. O(cap / 64).
    pub @get fn last(self) -> ?Int = todo

    /// Smallest set bit `>= i`, or `none`. O(cap / 64).
    pub fn next(self, i: Int) -> ?Int = todo

    /// Largest set bit `<= i`, or `none`. O(cap / 64).
    pub fn prev(self, i: Int) -> ?Int = todo

    /// Number of set bits strictly below `i`. O(cap / 64).
    pub fn count_below(self, i: Int) -> Int = todo

    /// Number of set bits in `r`. O(r.len / 64).
    pub fn count_range(self, r: Range<Int>) -> Int = todo

    /// The `k`-th set bit index (0-based), or `none`. O(cap / 64).
    pub fn nth(self, k: Int) -> ?Int = todo

    /// In-place bitwise OR (`self |= other`). O(cap / 64).
    pub fn union(mut self, other: DynBitset) = todo

    /// In-place bitwise AND (`self &= other`). O(cap / 64).
    pub fn intersect(mut self, other: DynBitset) = todo

    /// In-place set difference (`self &= ~other`). O(cap / 64).
    pub fn subtract(mut self, other: DynBitset) = todo

    /// In-place bitwise XOR (`self ^= other`). O(cap / 64).
    pub fn symmetric(mut self, other: DynBitset) = todo

    /// Whether `self` and `other` share a set bit. O(cap / 64).
    pub fn meets(self, other: DynBitset) -> Bool = todo

    /// Shift all bits up by `k` in place. O(cap / 64).
    pub fn shift_up(mut self, k: Int) = todo

    /// Shift all bits down by `k` in place. O(cap / 64).
    pub fn shift_down(mut self, k: Int) = todo

    /// In-place `self |= (self << k)`. O(cap / 64).
    pub fn shift_or(mut self, k: Int) = todo

    /// Yield set bit indices in ascending order. O(cap / 64 + len).
    pub fn each(self) -> yields Int = todo

    /// Yield set bit indices in descending order. O(cap / 64 + len).
    pub fn each_rev(self) -> yields Int = todo

    /// All subset sums in `0..cap` reachable from `weights`. O(cap sqrt(S) / 64).
    pub fn subset_sums(cap: Int, weights: List<Int>) -> DynBitset = todo

/// O(bits) per op. Dynamic binary bitwise XOR trie multiset over non-negative `bits`-bit integers (default 60 bits).
pub struct XorTrie:
    _bits: Int

    /// O(1). Create an empty `XorTrie` over `bits`-bit non-negative integers (default 60).
    pub fn new(bits: Int = 60) -> XorTrie = todo

    /// O(nodes). Reserve pool capacity for at least `nodes` trie nodes.
    pub fn reserve(mut self, nodes: Int) = todo

    /// O(1). Total number of stored integers (with multiplicity).
    pub @get fn len(self) -> Int = todo

    /// O(1). True when the trie holds no integers.
    pub @get fn is_empty(self) -> Bool = todo

    /// O(bits). Insert one copy of `x`.
    pub fn add(mut self, x: Int) = todo

    /// O(bits). Remove one copy of `x`; returns `false` if absent.
    pub fn remove(mut self, x: Int) -> Bool = todo

    /// O(bits). Whether `x` is present at least once.
    pub fn contains(self, x: Int) -> Bool = todo

    /// O(bits). Multiplicity of `x` currently stored.
    pub fn count(self, x: Int) -> Int = todo

    /// O(1). Apply a lazy global XOR mask (`y -> y ^ mask` for all stored elements).
    pub fn xor_all(mut self, mask: Int) = todo

    /// O(bits). Maximum value of `x ^ y` over stored `y`, or `none` if empty.
    pub fn max_xor(self, x: Int) -> ?Int = todo

    /// O(bits). Minimum value of `x ^ y` over stored `y`, or `none` if empty.
    pub fn min_xor(self, x: Int) -> ?Int = todo

    /// O(bits). Count of stored `y` with `(x ^ y) < limit`.
    pub fn count_xor_less(self, x: Int, limit: Int) -> Int = todo

    /// O(bits). The `k`-th smallest value of `x ^ y` over stored `y` (0-based), or `none` if `k >= len`.
    pub fn kth_xor(self, x: Int, k: Int) -> ?Int = todo

    /// O(bits). Smallest non-negative integer not present under XOR mask `x` (mex of `{y ^ x}`).
    pub fn mex_xor(self, x: Int = 0) -> Int = todo

    /// O(1). Remove all elements.
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
