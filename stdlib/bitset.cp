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
            if a[i]: b.insert(i)
        b

    pub fn has(self, i: D) -> Bool = ((w[i.to_idx / 64] >> (i.to_idx % 64)) & 1) == 1

    pub fn insert(mut self, i: D):
        w[i.to_idx / 64] |= bit(i.to_idx % 64)

    pub fn remove(mut self, i: D):
        w[i.to_idx / 64] &= bit(i.to_idx % 64) ^ -1

    pub fn flip(mut self, i: D):
        w[i.to_idx / 64] ^= bit(i.to_idx % 64)

    /// The number of positions in the set.
    pub @get fn count(self) -> Int = sum(popcount(x) for x in w)

    pub @get fn is_empty(self) -> Bool = self.count == 0

    pub fn union(mut self, o: Bitset<D>):
        for k in 0..w.len: w[k] |= o.w[k]

    pub fn intersect(mut self, o: Bitset<D>):
        for k in 0..w.len: w[k] &= o.w[k]

    pub fn subtract(mut self, o: Bitset<D>):
        for k in 0..w.len: w[k] &= o.w[k] ^ -1

    pub fn symmetric(mut self, o: Bitset<D>):
        for k in 0..w.len: w[k] ^= o.w[k]

    /// Do the two sets share a position? Stops at the first shared word.
    pub fn meets(self, o: Bitset<D>) -> Bool:
        for k in 0..w.len:
            if (w[k] & o.w[k]) != 0: return true
        false

    /// Move every position `k` places up; positions that leave `D` are dropped.
    /// This is the knapsack step: `reach.union(reach.shifted(weight))`.
    pub fn shifted(self, k: Int) -> Bitset<D>:
        var out = Bitset<D>.new()
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
    for i in [0, 1, 62, 63, 64, 127, 128, 199]: b.insert(P(i))
    assert b.count == 8
    assert b.has(P(63)) and b.has(P(64)) and not b.has(P(65))
    b.remove(P(63))
    b.flip(P(65))
    b.flip(P(0))
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
        assert u.has(p) == (x[p] or y[p])
        assert n.has(p) == (x[p] and y[p])
        assert d.has(p) == (x[p] and not y[p])
        assert s.has(p) == (x[p] != y[p])
    assert bx.meets(by) != n.is_empty
    assert bx.count == sum(1 for p in P if x[p])

@test
fn shifted_matches_moving_each_position():
    domain P = 0..190
    let x = [P] p => rand(0, 3) == 0
    let b = Bitset<P>.from(x)
    for k in [0, 1, 7, 63, 64, 65, 128, 189, 190, 400]:
        let s = b.shifted(k)
        for p in P:
            let from = p.to_idx - k
            assert s.has(p) == (from >= 0 and x[P.nth(max(from, 0))])

@test
fn subset_sum_by_shifting():
    domain S = 0..=100
    var reach = Bitset<S>()
    reach.insert(S(0))
    for wt in [7, 13, 29, 40]: reach.union(reach.shifted(wt))
    assert reach.has(S(20)) and reach.has(S(89)) and reach.has(S(49))
    assert not reach.has(S(1)) and not reach.has(S(30))
    assert reach.count == 16
