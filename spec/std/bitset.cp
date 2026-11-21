//! A dense set of positions of a domain, 64 per word.

/// One bit per position of `D`, in `ceil(K / 64)` words.
/// `Bitset<D>` is an explicit type you ask for; `[D]Bool` stays one byte
/// per element and never turns into this behind your back.
/// Single-bit operations are O(1); whole-set operations are O(K / 64).
pub struct Bitset<D>:
    w: List<U64>

    /// The empty set. O(K / 64).
    pub fn new() -> Bitset<D>:
        Bitset(List.filled((D.K + 63) / 64, U64(0)))

    /// The empty set, matching `Subset<D>.empty`. O(K / 64).
    pub @get fn empty() -> Bitset<D> = Bitset<D>.new()

    /// The full set containing every position of `D`, matching `Subset<D>.full`. O(K / 64).
    pub @get fn full() -> Bitset<D>:
        var b = Bitset(List.filled((D.K + 63) / 64, ~U64(0)))
        b.trim()
        b

    /// The singleton set `{i}`, matching `Subset<D>.single(i)`. O(K / 64).
    pub fn single(i: D) -> Bitset<D>:
        var b = Bitset<D>.new()
        b.add(i)
        b

    /// The set of positions where `a` is true. O(D.K).
    pub fn from(a: [D]Bool) -> Bitset<D>:
        var b = Bitset<D>.new()
        for i in D:
            if a[i]: b.add(i)
        b

    /// Build a bitset from a list or generator of positions `D`. O(D.K / 64 + xs.len).
    pub fn of(xs: List<D>) -> Bitset<D>:
        var b = Bitset<D>.new()
        for i in xs: b.add(i)
        b

    /// Whether position `i` is set. O(1).
    pub fn contains(self, i: D) -> Bool = ((w[i.to_idx / 64] >> (i.to_idx % 64)) & U64(1)) == U64(1)

    /// Set the bit at position `i`. O(1).
    pub fn add(mut self, i: D):
        w[i.to_idx / 64] |= bit(i.to_idx % 64)

    /// Clear the bit at position `i`. O(1).
    pub fn remove(mut self, i: D):
        w[i.to_idx / 64] &= ~bit(i.to_idx % 64)

    /// Flip the bit at position `i`. O(1).
    pub fn toggle(mut self, i: D):
        w[i.to_idx / 64] ^= bit(i.to_idx % 64)

    /// Copy of `self` with `i` added. O(K / 64).
    pub fn with(self, i: D) -> Bitset<D>:
        var out = Bitset(w)
        out.add(i)
        out

    /// Copy of `self` with `i` removed. O(K / 64).
    pub fn without(self, i: D) -> Bitset<D>:
        var out = Bitset(w)
        out.w[i.to_idx / 64] &= ~bit(i.to_idx % 64)
        out

    /// Copy of `self` with `i` added if `on` or removed otherwise, matching `Subset<D>.set`. O(K / 64).
    pub fn set(self, i: D, on: Bool) -> Bitset<D>:
        if on: self.with(i) else: self.without(i)

    /// Complement of `self` within `D`, matching `Subset<D>.complement`. O(K / 64).
    pub @get fn complement(self) -> Bitset<D>:
        var out = Bitset<D>.new()
        for k in 0..w.len: out.w[k] = ~w[k]
        out.trim()
        out

    /// Remove every position from the set. O(K / 64).
    pub fn clear(mut self):
        for k in 0..w.len: w[k] = U64(0)

    /// [rename] First position in this set satisfying `pred`, or `none` (replaces `find`). O(K / 64 + visited).
    pub fn first_where(self, pred: D -> Bool) -> ?D:
        for k in 0..w.len:
            var x = w[k]
            while x != U64(0):
                let b = ctz(x)
                let v = D.nth(64 * k + b)
                if pred(v): return v
                x &= ~bit(b)
        none

    /// True when some position in this set satisfies `pred`. O(K / 64 + visited).
    pub fn any(self, pred: D -> Bool) -> Bool = self.first_where(pred) != none
    /// The number of positions in the set.
    pub @get fn len(self) -> Int = sum(popcount(x) for x in w)

    /// The same number as `len`, under the name C++ gives it.
    pub @get fn count(self) -> Int = self.len

    /// Whether the set has no members; stops at the first non-zero word. O(K / 64).
    pub @get fn is_empty(self) -> Bool:
        for x in w:
            if x != 0: return false
        true

    /// The smallest position in the set, or `none`. O(K / 64).
    pub @get fn first(self) -> ?D:
        for k in 0..w.len:
            if w[k] != 0: return D.nth(64 * k + ctz(w[k]))
        none

    /// The largest position in the set, or `none`. O(K / 64).
    pub @get fn last(self) -> ?D:
        for k in (0..w.len).rev:
            if w[k] != 0: return D.nth(64 * k + msb(w[k]))
        none

    /// Replace `self` with `self | o` in place. O(|D| / 64).
    pub fn union(mut self, o: Bitset<D>):
        for k in 0..w.len: w[k] |= o.w[k]

    /// Replace `self` with `self & o` in place. O(|D| / 64).
    pub fn intersect(mut self, o: Bitset<D>):
        for k in 0..w.len: w[k] &= o.w[k]

    /// Clear every bit set in `o` from `self` in place. O(|D| / 64).
    pub fn subtract(mut self, o: Bitset<D>):
        for k in 0..w.len: w[k] &= ~o.w[k]

    /// Replace `self` with `self ^ o` in place. O(|D| / 64).
    pub fn symmetric(mut self, o: Bitset<D>):
        for k in 0..w.len: w[k] ^= o.w[k]

    /// `a | b`: the positions in either set.
    pub fn bitor(self, o: Bitset<D>) -> Bitset<D>:
        var out = Bitset<D>.new()
        for k in 0..w.len: out.w[k] = w[k] | o.w[k]
        out

    /// `a & b`: the positions in both sets.
    pub fn bitand(self, o: Bitset<D>) -> Bitset<D>:
        var out = Bitset<D>.new()
        for k in 0..w.len: out.w[k] = w[k] & o.w[k]
        out

    /// `a ^ b`: the positions in exactly one of the sets.
    pub fn bitxor(self, o: Bitset<D>) -> Bitset<D>:
        var out = Bitset<D>.new()
        for k in 0..w.len: out.w[k] = w[k] ^ o.w[k]
        out

    /// Do the two sets share a position? Stops at the first shared word.
    pub fn meets(self, o: Bitset<D>) -> Bool:
        for k in 0..w.len:
            if (w[k] & o.w[k]) != 0: return true
        false

    /// Move every position `k` places up (down for a negative `k`); positions that leave `D` are dropped.
    /// O(K / 64) for both positive and negative `k`.
    pub fn shifted(self, k: Int) -> Bitset<D>:
        var out = Bitset<D>.new()
        if k < 0:
            let d = -k
            let (words, bits) = (d / 64, d % 64)
            if words >= w.len: return out
            for i in 0..w.len - words:
                var x = w[i + words] >> bits
                if bits > 0 and i + words + 1 < w.len:
                    x |= w[i + words + 1] << (64 - bits)
                out.w[i] = x
            return out
        let (words, bits) = (k / 64, k % 64)
        for i in (words..w.len).rev:
            var x = w[i - words] << bits
            if bits > 0 and i > words: x |= w[i - words - 1] >> (64 - bits)
            out.w[i] = x
        out.trim()
        out

    /// Shift every position `k >= 0` places up in place; bits past `D` drop. O(K / 64).
    pub fn shift_up(mut self, k: Int):
        assert k >= 0
        if k == 0: return
        let (words, bits) = (k / 64, k % 64)
        if words >= w.len:
            self.clear()
            return
        for i in (words..w.len).rev:
            var x = w[i - words] << bits
            if bits > 0 and i > words: x |= w[i - words - 1] >> (64 - bits)
            w[i] = x
        for i in 0..words: w[i] = U64(0)
        self.trim()

    /// In-place `self |= self << k` for `k >= 0`, with zero allocation. O(K / 64).
    pub fn shift_or(mut self, k: Int):
        assert k >= 0
        if k == 0: return
        let (words, bits) = (k / 64, k % 64)
        if words >= w.len: return
        for i in (words..w.len).rev:
            var x = w[i - words] << bits
            if bits > 0 and i > words: x |= w[i - words - 1] >> (64 - bits)
            w[i] |= x
        self.trim()

    /// The positions in the set, in increasing order.
    pub fn each(self) -> yields D:
        for k in 0..w.len:
            var x = w[k]
            while x != 0:
                let b = ctz(x)
                yield D.nth(64 * k + b)
                x &= ~bit(b)

    /// The first position in the set at or after `i`, or `none`. O(K / 64).
    pub fn next(self, i: D) -> ?D:
        var k = i.to_idx / 64
        var x = w[k] & ~low(i.to_idx % 64)
        while true:
            if x != 0: return D.nth(64 * k + ctz(x))
            k += 1
            if k == w.len: return none
            x = w[k]

    /// The last position in the set strictly before `i`, or `none`. O(K / 64).
    pub fn prev(self, i: D) -> ?D:
        let idx = i.to_idx
        if idx == 0: return none
        var k = idx / 64
        var x = w[k] & low(idx % 64)
        while true:
            if x != 0: return D.nth(64 * k + msb(x))
            if k == 0: return none
            k -= 1
            x = w[k]

    /// Number of positions in the set strictly before `i`. O(K / 64).
    pub fn count_below(self, i: D) -> Int:
        let idx = i.to_idx
        let (words, bits) = (idx / 64, idx % 64)
        var ans = 0
        for k in 0..words: ans += popcount(w[k])
        if bits > 0 and words < w.len: ans += popcount(w[words] & low(bits))
        ans

    /// All subset sums in `0..D.K` reachable from non-negative `weights`.
    /// Groups duplicate weights with binary carries so the work is O(K sqrt(S) / 64).
    pub fn subset_sums(weights: List<Int>) -> Bitset<D>:
        var reach = Bitset<D>.new()
        if D.K == 0: return reach
        reach.add(D.nth(0))
        var cnt = List.filled(D.K, 0)
        for w in weights:
            if w > 0 and w < D.K: cnt[w] += 1
        for w in 1..D.K:
            let c = cnt[w]
            if c > 2:
                let carry = (c - 1) / 2
                cnt[w] -= 2 * carry
                if 2 * w < D.K: cnt[2 * w] += carry
            for _ in 0..cnt[w]: reach.shift_or(w)
        reach

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

    fn trim(mut self):
        let full_words = (D.K + 63) / 64
        for k in full_words..w.len: w[k] = U64(0)
        if D.K % 64 != 0 and full_words > 0:
            w[full_words - 1] &= low(D.K % 64)

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

    /// Set bit `i` to 1. O(1).
    pub fn add(mut self, i: Int) = todo

    /// Clear bit `i` to 0. O(1).
    pub fn remove(mut self, i: Int) = todo

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
    fn sub(a, b):
        var out = Bitset<D>.new()
        for k in 0..a.w.len: out.w[k] = a.w[k] & ~b.w[k]
        out

/// Index of the highest set bit in non-zero `x` (0..64).
fn msb(x: U64) -> Int = bit_width(x) - 1

impl Show for Bitset<D>:
    fn show(a):
        var out = ""
        for v in a.each():
            if not out.is_empty: out += " "
            out += "{v}"
        out
/// The word with only bit `k` set.
fn bit(k: Int) -> U64 = U64(1) << k

/// The word with its low `n` bits set, for `n` in 0..=64.
fn low(n: Int) -> U64 = if n == 64: ~U64(0) else: (U64(1) << n) - U64(1)

@test
fn single_bits_across_word_borders():
    domain P = 0..200
    var b = Bitset<P>()
    for i in [0, 1, 62, 63, 64, 127, 128, 199]: b.add(P(i))
    assert b.len == 8 and b.count == 8
    assert b.contains(P(63)) and b.contains(P(64)) and not b.contains(P(65))
    b.remove(P(63))
    b.toggle(P(65))
    b.toggle(P(0))
    assert [p.to_idx for p in b.each()] == [1, 62, 64, 65, 127, 128, 199]
    assert b.next(P(2)) == P(62) and b.next(P(129)) == P(199) and b.next(P(64)) == P(64)
    b.remove(P(199))
    assert b.next(P(129)) == none

@test
fn set_algebra_matches_bool_arrays():
    domain P = 0..150
    let x = [P] p => rand(0, 2) == 0
    let y = [P] p => rand(0, 2) == 0
    let (bx, by) = (Bitset<P>.from(x), Bitset<P>.from(y))
    var u = copy(bx)
    u.union(by)
    var n = copy(bx)
    n.intersect(by)
    var d = copy(bx)
    d.subtract(by)
    var s = copy(bx)
    s.symmetric(by)
    for p in P:
        assert u.contains(p) == (x[p] or y[p])
        assert n.contains(p) == (x[p] and y[p])
        assert d.contains(p) == (x[p] and not y[p])
        assert s.contains(p) == (x[p] != y[p])
    assert bx.meets(by) != n.is_empty
    assert bx.len == sum(1 for p in P if x[p])

@test
fn shifted_matches_moving_each_position():
    domain P = 0..190
    let x = [P] p => rand(0, 3) == 0
    let b = Bitset<P>.from(x)
    for k in [0, 1, 7, 63, 64, 65, 128, 189, 190, 400]:
        let s = b.shifted(k)
        for p in P:
            let from = p.to_idx - k
            assert s.contains(p) == (from >= 0 and x[P.nth(max(from, 0))])

@test
fn subset_sum_by_shifting():
    domain S = 0..=100
    var reach = Bitset<S>()
    reach.add(S(0))
    for wt in [7, 13, 29, 40]: reach.union(reach.shifted(wt))
    assert reach.contains(S(20)) and reach.contains(S(89)) and reach.contains(S(49))
    assert not reach.contains(S(1)) and not reach.contains(S(30))
    assert reach.len == 16

@test
fn membership_and_a_signed_shift():
    domain D = 0..130
    var s = Bitset<D>()
    for x in [0, 3, 64, 129]: s.add(D(x))
    assert D(64) in s and D(65) not in s and s.len == 4
    s.toggle(D(3))
    s.toggle(D(5))
    assert [i.to_idx for i in s.shifted(-3).each()] == [2, 61, 126]
    assert [i.to_idx for i in s.shifted(2).each()] == [2, 7, 66]

@test
fn full_contains_every_position_and_nothing_past_end():
    for sz in [1, 63, 64, 65, 130]:
        domain D = 0..sz
        let f = Bitset<D>.full()
        assert f.len == sz
        for x in D: assert x in f
        assert (f & Bitset<D>.new()).is_empty

@test
fn nav_helpers_and_in_place_shifts():
    domain P = 0..195
    var b = Bitset<P>.new()
    assert b.first == none and b.last == none
    for idx in [0, 63, 64, 127, 128, 194]: b.add(P(idx))
    assert b.first == P(0) and b.last == P(194)
    assert b.prev(P(0)) == none
    assert b.prev(P(64)) == P(63)
    assert b.prev(P(128)) == P(127)
    assert b.prev(P(194)) == P(128)
    assert b.count_below(P(0)) == 0
    assert b.count_below(P(64)) == 2
    assert b.count_below(P(128)) == 4
    assert b.count_below(P(194)) == 5
    assert b.with(P(5)).contains(P(5)) and not b.contains(P(5))
    assert not b.without(P(64)).contains(P(64)) and b.contains(P(64))
    assert b.first_where(x => x.to_idx > 100) == P(127)
    assert b.first_where(x => x.to_idx > 200) == none

    // Word-level negative shifts across word boundaries.
    for k in [1, 7, 63, 64, 65, 128, 194, 300]:
        let s = b.shifted(-k)
        for p in P:
            let src = p.to_idx + k
            assert s.contains(p) == (src < 195 and b.contains(P(src)))

    // shift_up and shift_or agree with shifted.
    var u = Bitset<P>.new()
    for idx in [0, 1, 63, 64, 129]: u.add(P(idx))
    let expected_up = u.shifted(65)
    u.shift_up(65)
    for p in P: assert u.contains(p) == expected_up.contains(p)

    var o = Bitset<P>.new()
    o.add(P(0))
    for w in [3, 64, 65, 17]:
        let exp = o | o.shifted(w)
        o.shift_or(w)
        for p in P: assert o.contains(p) == exp.contains(p)
    o.clear()
    assert o.is_empty and o.first == none and o.last == none

@test
fn subset_sums_matches_naive_with_duplicates():
    domain S = 0..=150
    let ws = [1, 1, 1, 1, 1, 2, 2, 2, 3, 3, 3, 3, 7, 7, 15, 40, 40, 200, 0]
    let fast = Bitset<S>.subset_sums(ws)
    var naive = Bitset<S>.new()
    naive.add(S(0))
    for w in ws:
        if w > 0: naive.union(naive.shifted(w))
    for s in S: assert fast.contains(s) == naive.contains(s)

@test
fn subset_parity_constructors_and_complement():
    domain P = 0..70
    let e = Bitset<P>.empty
    let f = Bitset<P>.full
    assert e.is_empty and f.len == 70
    let s = Bitset<P>.single(P(64))
    assert s.len == 1 and P(64) in s and s.first == P(64) and s.last == P(64)
    let c = s.complement
    assert c.len == 69 and P(64) not in c and P(0) in c and P(69) in c
    let s2 = s.set(P(3), true).set(P(64), false)
    assert s2.len == 1 and P(3) in s2 and P(64) not in s2
    let evens = Bitset<P>.of(p for p in P if p.to_idx % 2 == 0)
    assert evens.len == 35 and P(0) in evens and P(1) not in evens and P(68) in evens
