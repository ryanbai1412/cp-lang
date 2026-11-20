//! A dense set of positions of a domain, 64 per word.

/// One bit per position of `D`, in `ceil(K / 64)` words.
/// `Bitset<D>` is an explicit type you ask for; `[D]Bool` stays one byte
/// per element and never turns into this behind your back.
/// Single-bit operations are O(1); whole-set operations are O(K / 64).
pub struct Bitset<D>:
    w: List<Int>

    /// The empty set.
    pub fn new() -> Bitset<D>:
        Bitset(List.filled((D.K + 63) / 64, 0))

    /// The set of positions where `a` is true.
    pub fn from(a: [D]Bool) -> Bitset<D>:
        var b = Bitset<D>.new()
        for i in D:
            if a[i]: b.add(i)
        b

    pub fn contains(self, i: D) -> Bool = ((w[i.to_idx / 64] >> (i.to_idx % 64)) & 1) == 1

    pub fn add(mut self, i: D):
        w[i.to_idx / 64] |= bit(i.to_idx % 64)

    pub fn remove(mut self, i: D):
        w[i.to_idx / 64] &= bit(i.to_idx % 64) ^ -1

    pub fn toggle(mut self, i: D):
        w[i.to_idx / 64] ^= bit(i.to_idx % 64)

    /// The number of positions in the set.
    pub @get fn len(self) -> Int = sum(popcount(x) for x in w)

    /// The same number as `len`, under the name C++ gives it.
    pub @get fn count(self) -> Int = self.len

    pub @get fn is_empty(self) -> Bool = self.len == 0

    pub fn union(mut self, o: Bitset<D>):
        for k in 0..w.len: w[k] |= o.w[k]

    pub fn intersect(mut self, o: Bitset<D>):
        for k in 0..w.len: w[k] &= o.w[k]

    pub fn subtract(mut self, o: Bitset<D>):
        for k in 0..w.len: w[k] &= o.w[k] ^ -1

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
    /// This is the knapsack step: `reach.union(reach.shifted(weight))`.
    pub fn shifted(self, k: Int) -> Bitset<D>:
        var out = Bitset<D>.new()
        if k < 0:
            // towards lower positions: the ones that fall below the first are dropped
            for i in self.each():
                if let j = D.try_at(i.to_idx + k): out.add(j)
            return out
        let (words, bits) = (k / 64, k % 64)
        for i in (words..w.len).rev:
            var x = shl(w[i - words], bits)
            if bits > 0 and i > words: x |= shr(w[i - words - 1], 64 - bits)
            out.w[i] = x
        out.trim()
        out

    /// The positions in the set, in increasing order.
    pub fn each(self) -> yields D:
        for k in 0..w.len:
            var x = w[k]
            while x != 0:
                let b = ctz(x)
                yield D.nth(64 * k + b)
                x &= bit(b) ^ -1

    /// The first position in the set at or after `i`, or `none`.
    pub fn next(self, i: D) -> ?D:
        var k = i.to_idx / 64
        var x = w[k] & (low(i.to_idx % 64) ^ -1)
        while true:
            if x != 0: return D.nth(64 * k + ctz(x))
            k += 1
            if k == w.len: return none
            x = w[k]

    fn trim(mut self):
        if D.K % 64 != 0 and w.len > 0:
            w[w.len - 1] &= low(D.K % 64)

// `Int` is a signed 64-bit word, so the helpers below spell out the three
// places where bit 63 needs care. They compile to single instructions.

/// `a - b`: the positions in `a` that are not in `b`.
impl Sub for Bitset<D>:
    fn sub(a, b):
        var out = Bitset<D>.new()
        for k in 0..a.w.len: out.w[k] = a.w[k] & (b.w[k] ^ -1)
        out

/// The word with only bit `k` set.
fn bit(k: Int) -> Int = if k == 63: Int.MIN else: 1 << k

/// The word with its low `n` bits set, for `n` in 0..=64.
fn low(n: Int) -> Int = if n == 64: -1 else: if n == 63: Int.MAX else: (1 << n) - 1

/// Shift left, dropping the bits that leave the word.
fn shl(x: Int, k: Int) -> Int:
    if k == 0: return x
    let top = if ((x >> (63 - k)) & 1) == 1: Int.MIN else: 0
    ((x & low(63 - k)) << k) | top

/// Shift right, filling with zeros.
fn shr(x: Int, k: Int) -> Int = if k == 0: x else: (x >> k) & low(64 - k)

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
