//! Random numbers from a seed: the same seed gives the same numbers in `cpl run`
//! and in the built C++. `Rng(seed)` is a splitmix64 stream: O(1) per draw.

pub struct Rng(s: U64):
    /// O(1). A stream seeded by `seed`.
    pub fn new(seed: Int) -> Rng = todo
    /// O(1). The next 64 random bits.
    pub fn next(mut self) -> U64 = todo
    /// O(1). Uniform in `r`; traps on an empty range.
    pub fn int(mut self, r: Range<Int>) -> Int = todo
    /// O(1). Uniform in [0, 1).
    pub fn float(mut self) -> F64 = todo
    /// O(1). True with probability `p`.
    pub fn chance(mut self, p: F64) -> Bool = todo
    /// O(1). A uniformly random element of `xs`; none when empty.
    pub fn pick<S: Indexed, T>(mut self, xs: S) -> ?T = todo
    /// O(n). Shuffle `a` in place (Fisher-Yates).
    pub fn shuffle<S: IndexedMut>(mut self, mut a: S) = todo
    /// O(n). A uniformly random permutation of `D`.
    pub fn perm<D>(mut self) -> [D]D = todo
    /// O(|from|). `k` distinct elements of `from`, uniformly, in random order.
    pub fn sample<S: Indexed, T>(mut self, k: Int, from: S) -> List<T> = todo
