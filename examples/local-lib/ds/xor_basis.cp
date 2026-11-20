//! A linear basis over GF(2): the set of values reachable as the xor of a subset.

/// A basis of non-negative `Int`s under xor. Every operation is O(62).
pub struct XorBasis:
    /// `row[b]` is zero, or a basis vector whose highest set bit is `b`.
    row: List<Int>
    rank: Int

    pub fn new() -> XorBasis = XorBasis(List.filled(62, 0), 0)

    /// `x` with every basis vector that lowers it xored in: zero exactly when `x` is reachable.
    pub fn reduce(self, x: Int) -> Int:
        var y = x
        for b in (0..62).rev:
            if (y >> b) % 2 == 1: y ^= row[b]
        y

    /// Add `x`. `true` if it made more values reachable.
    pub fn add(mut self, x: Int) -> Bool:
        var y = x
        for b in (0..62).rev:
            if (y >> b) % 2 == 0: continue
            if row[b] == 0:
                row[b] = y
                rank += 1
                return true
            y ^= row[b]
        false

    /// Is `x` the xor of some subset of the values added?
    pub fn contains(self, x: Int) -> Bool = self.reduce(x) == 0

    /// The largest value of `start ^ (xor of a subset)`.
    pub fn max_xor(self, start: Int = 0) -> Int:
        var y = start
        for b in (0..62).rev: y = max(y, y ^ row[b])
        y

    /// The number of distinct reachable values: `2 ** rank`.
    pub @get fn len(self) -> Int = 1 << rank

@test
fn matches_trying_every_subset():
    domain Val = 0..64
    for _ in 0..40:
        let xs = [rand(0, 63) for _ in 0..rand(0, 6)]
        var reach: [Val]Bool = false
        for mask in 0..(1 << xs.len):
            var v = 0
            for i in 0..xs.len:
                if (mask >> i) % 2 == 1: v ^= xs[i]
            reach[Val(v)] = true
        var b = XorBasis()
        for x in xs: b.add(x)
        for v in Val: assert (v.to_idx in b) == reach[v]
        assert b.len == count(reach[v] for v in Val)
        let start = rand(0, 63)
        assert b.max_xor(start) == max(start ^ v.to_idx for v in Val if reach[v])

@test
fn add_reports_dependence():
    var b = XorBasis()
    assert b.add(5) and b.add(3) and not b.add(6) and not b.add(0)
    assert b.rank == 2 and b.max_xor() == 6
