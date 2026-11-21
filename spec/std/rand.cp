//! Random numbers from a seed: the same seed gives the same numbers in `cpl run` and in the built C++.

/// A xorshift64* generator. Every draw is O(1); the state is one word.
pub struct Rng(s: U64):
    /// A generator started from `seed`; any seed works, also 0.
    pub fn new(seed: Int) -> Rng = Rng(U64(seed) ^ U64(-7046029254386353131))

    /// The next 64 random bits.
    pub fn next(mut self) -> U64:
        s ^= s >> 12
        s ^= s << 25
        s ^= s >> 27
        s * U64(2685821657736338717)

    /// A whole number in `lo..=hi`, each about equally likely.
    pub fn int(mut self, lo: Int, hi: Int) -> Int:
        assert lo <= hi, "`int` needs `lo <= hi`"
        // `hi - lo + 1` is up to 2 ** 64, past an `Int`, when `lo` is negative.
        if lo >= 0: return lo + Int(self.next() % (U64(hi - lo) + U64(1)))
        let width = I128(hi) - I128(lo) + I128(1)
        Int(I128(lo) + I128(self.next()) % width)

    /// A number in `0.0..1.0`.
    /// A fair coin flip. O(1).
    pub fn coin(mut self) -> Bool = todo

    pub fn float(mut self) -> F64 = F64(Int(self.next() >> 11)) / 9007199254740992.0

    /// `a` in a random order, in place. Works on any `IndexedMut` (`List`, `[D]T`, `Str`). O(n).
    pub fn shuffle<S: IndexedMut>(mut self, mut a: S):
        for i in (1..a.len).rev: a.swap_idx(i, self.int(0, i))

@test
fn the_same_seed_gives_the_same_numbers():
    var (a, b) = (Rng.new(7), Rng.new(7))
    assert all(a.next() == b.next() for _ in 0..10)
    var r = Rng.new(0)
    assert all(3 <= x and x <= 5 for x in [r.int(3, 5) for _ in 0..100])
    assert all(0.0 <= x and x < 1.0 for x in [r.float() for _ in 0..100])
    var xs = [i for i in 0..20]
    r.shuffle(mut xs)
    assert sorted(xs) == [i for i in 0..20] and xs != [i for i in 0..20]

@test
fn int_covers_the_widest_ranges():
    var rng = Rng.new(7)
    for _ in 0..200:
        assert rng.int(0, Int.MAX) >= 0
        assert rng.int(Int.MIN, -1) < 0
        let x = rng.int(Int.MIN, Int.MAX)
        assert Int.MIN <= x and x <= Int.MAX
        let y = rng.int(-3, 3)
        assert -3 <= y and y <= 3
    assert rng.int(Int.MAX, Int.MAX) == Int.MAX
    assert rng.int(Int.MIN, Int.MIN) == Int.MIN

