//! Random numbers from a seed: the same seed gives the same numbers in `cpl run`
//! and in the built C++. `Rng(seed)` is a splitmix64 stream: O(1) per draw.

pub struct Rng(s: U64):
    /// A deterministic stream of random bits seeded by `seed`.
    ///
    /// The state is `U64(seed)` (two's complement for negative seeds) and each `next()` is one
    /// splitmix64 step, so equal seeds give equal streams on every build and target. There is no
    /// time-seeded constructor: tests and stress runs replay exactly.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var r = Rng(0); r.next() == 0xE220A8397B1DCDAF`
    /// example: `var a = Rng(7); var b = Rng(7); a.next() == b.next()`
    pub fn new(seed: Int) -> Rng = Rng(U64(seed) ^ U64(-7046029254386353131))
    /// The next 64 random bits: one splitmix64 step.
    ///
    /// The state advances by `0x9E3779B97F4A7C15` (wrapping) and the result is the splitmix64 mix of
    /// the new state: `z ^= z >> 30; z *= 0xBF58476D1CE4E5B9; z ^= z >> 27; z *= 0x94D049BB133111EB;
    /// z ^= z >> 31`, all wrapping. Every other `Rng` method draws its bits from `next`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var r = Rng(0); r.next(); r.next() == 0x6E789E6AA1B965F4`
    /// example: `var r = Rng(1); r.next() == 10451216379200822465`
    pub fn next(mut self) -> U64:
        s ^= s >> 12
        s ^= s << 25
        s ^= s >> 27
        s * U64(2685821657736338717)
    /// A uniformly random integer in `lo..=hi`, both ends inclusive, from one `next()`.
    ///
    /// With `n = hi - lo + 1`, the result is `lo + ((U128(next()) * n) >> 64)`: Lemire's
    /// multiply-high map, which uses exactly one draw (bias at most `n / 2^64`).
    ///
    /// cost: O(1)
    /// empty: traps
    /// traps: when `lo > hi`
    /// example: `var r = Rng(1); let x = r.int(1, 6); 1 <= x and x <= 6`
    /// example: `var r = Rng(5); r.int(4, 4) == 4`
    pub fn int(mut self, lo: Int, hi: Int) -> Int:
        assert lo <= hi, "`int` needs `lo <= hi`"
        // `hi - lo + 1` is up to 2 ** 64, past an `Int`, when `lo` is negative.
        if lo >= 0: return lo + Int(self.next() % (U64(hi - lo) + U64(1)))
        let width = I128(hi) - I128(lo) + I128(1)
        Int(I128(lo) + I128(self.next()) % width)
    /// A uniformly random `F64` in `[0, 1)`: the top 53 bits of `next()` times `2^-53`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var r = Rng(5); let x = r.float(); 0.0 <= x and x < 1.0`
    pub fn float(mut self) -> F64 = F64(Int(self.next() >> 11)) / 9007199254740992.0
    /// True with probability `p`: `float() < p`.
    ///
    /// `p <= 0` is always false and `p >= 1` always true; both still consume one draw so the
    /// stream stays aligned whatever `p` is.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var r = Rng(3); r.chance(1.0) == true`
    /// example: `var r = Rng(3); r.chance(0.0) == false`
    pub fn chance(mut self, p: F64) -> Bool = todo
    /// A uniformly random element of `xs`: `xs` at position `int(0..xs.len)`.
    ///
    /// The position is zero-based, so any key type works. An empty `xs` returns `None` without
    /// drawing.
    ///
    /// cost: O(1)
    /// empty: `None`, and no draw
    /// traps: never
    /// example: `var r = Rng(1); r.pick([7]) == Some(7)`
    /// example: `var r = Rng(1); let x = r.pick([4, 5, 6]).unwrap(); 4 <= x and x <= 6`
    /// example: `let e: List<Int> = []; var r = Rng(1); r.pick(e) == None`
    pub fn pick<S: Indexed, T>(mut self, xs: S) -> ?T = todo
    /// Shuffle `a` in place, uniformly (Fisher-Yates).
    ///
    /// For `i` from `a.len - 1` down to `1`, swap positions `i` and `int(0..=i)`. Positions are
    /// zero-based, so this works on any `IndexedMut`: a `List`, a `[D]T` over any domain, a `Str`.
    /// Keys stay where they are; only the values move.
    ///
    /// cost: O(n)
    /// empty: no change, and no draw
    /// traps: never
    /// example: `var r = Rng(2); var a = [3, 1, 2]; r.shuffle(mut a); a.sorted() == [1, 2, 3]`
    /// example: `var r = Rng(2); var a = [9]; r.shuffle(mut a); a == [9]`
    pub fn shuffle<S: IndexedMut>(mut self, mut a: S):
        for i in (1..a.len).rev: a.swap_idx(i, self.int(0, i))
    /// A uniformly random permutation of the domain `D`, as a `[D]D`.
    ///
    /// It is the identity `[D]D` shuffled with `shuffle`, so a seed fixes the result. Every element
    /// of `D` appears exactly once among the values.
    ///
    /// cost: O(n)
    /// order: keyed by `D`; the value at each key is uniformly random, each element of `D` once
    /// traps: never
    /// example: `domain V = 0..4; var r = Rng(4); let p = r.perm<V>(); p.distinct().len == 4`
    pub fn perm<D>(mut self) -> [D]D = todo
    /// `k` elements of `from` at distinct positions, chosen uniformly, in random order.
    ///
    /// It runs a partial Fisher-Yates over a copy of the positions: for `i` in `0..k`, swap
    /// positions `i` and `int(i..n)`, then return the values at the first `k` positions. Equal
    /// values at different positions can both be chosen.
    ///
    /// cost: O(|from|)
    /// empty: `[]` when `k == 0`
    /// order: the order drawn (random); a new `List`, keyed from 0
    /// traps: when `k` is not in `0..=from.len`
    /// example: `var r = Rng(9); r.sample(3, from: [5, 6, 7]).sorted() == [5, 6, 7]`
    /// example: `var r = Rng(9); r.sample(0, from: [5, 6, 7]) == []`
    /// example: `var r = Rng(9); r.sample(2, from: [1, 1, 1]) == [1, 1]`
    pub fn sample<S: Indexed, T>(mut self, k: Int, from: S) -> List<T> = todo
