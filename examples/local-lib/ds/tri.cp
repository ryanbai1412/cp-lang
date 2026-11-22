//! Triangular tables: `[Tri<D>]T` holds one cell per pair `l <= r` of positions of `D`
//! and nothing else, which is half the memory of a `[D, D]T` for an interval DP or a
//! symmetric matrix.

/// A pair of positions `l <= r` of `D`, usable as an index: a `[Tri<D>]T` has
/// `K * (K + 1) / 2` cells for `K = D.K`. The cells are stored row by row for `r`, so the
/// intervals ending at one `r` sit side by side.
pub struct Tri<D>(l: D, r: D):
    /// The pair `(a, b)`. `a` must not be after `b`.
    pub fn new(a: D, b: D) -> Tri<D>:
        assert a <= b
        Tri(a, b)

    /// The pair in either order, for a symmetric table: `sym(a, b) == sym(b, a)`.
    pub fn sym(a: D, b: D) -> Tri<D> = if a <= b: Tri(a, b) else: Tri(b, a)

    /// Every pair, in the order of the cells.
    pub fn pairs() -> List<Tri<D>> = [Tri<D>.from_idx(i) for i in 0..D.K * (D.K + 1) / 2]

    /// The number of positions from `l` to `r`, both included.
    pub @get fn len(self) -> Int = r.to_idx - l.to_idx + 1

    /// Every pair with exactly `len` positions, by increasing `l`: the order an
    /// interval DP fills its table in, one `len` after another.
    pub fn of_len(len: Int) -> List<Tri<D>>:
        [Tri(D.from_idx(i)!, D.from_idx(i + len - 1)!) for i in 0..=D.K - len]

    /// The two parts of every split `l..=m`, `m+1..=r`, by increasing `m`.
    pub @get fn splits(self) -> List<(Tri<D>, Tri<D>)>:
        [(Tri(l, D.from_idx(m)!), Tri(D.from_idx(m + 1)!, r)) for m in l.to_idx..r.to_idx]

impl Index for Tri<D>:
    let K = D.K * (D.K + 1) / 2
    fn to_idx(self) -> Int = r.to_idx * (r.to_idx + 1) / 2 + l.to_idx
    fn from_idx(i: Int) -> Tri<D>:
        // the largest r with r * (r + 1) / 2 <= i
        let r = (isqrt(8 * i + 1) - 1) / 2
        Tri(D.from_idx(i - r * (r + 1) / 2)!, D.from_idx(r)!)

@test
fn every_pair_has_its_own_cell():
    for n in 1..=9:
        domain D = 1..=n
        assert Tri<D>.K == n * (n + 1) / 2
        var seen: [Tri<D>]Int = 0
        var at = 0
        // row by row for r, then by l: exactly the order of the cells
        for r in D:
            for l in D:
                if l > r: continue
                let t = Tri<D>(l, r)
                assert t.to_idx() == at and Tri<D>.from_idx(at) == t
                seen[t] += 1
                at += 1
        assert at == Tri<D>.K and Tri<D>.pairs().all(t => seen[t] == 1)

@test
fn pairs_by_length_and_their_splits():
    domain D = 0..6
    var total = 0
    for len in 1..=6:
        let ts = Tri<D>.of_len(len)
        assert ts.len == 7 - len and ts.all(t => t.len == len)
        total += ts.len
        for t in ts:
            assert t.splits.len == len - 1
            for (a, b) in t.splits:
                assert a.l == t.l and b.r == t.r and a.len + b.len == len and a.r.to_idx + 1 == b.l.to_idx
    assert total == Tri<D>.K
    assert Tri<D>.sym(D(4), D(1)) == Tri<D>(D(1), D(4)) and Tri<D>.sym(D(2), D(2)).len == 1

@test
fn a_symmetric_table_matches_the_square_one():
    domain V = 1..=7
    for _ in 0..20:
        var square: [V, V]Int = 0
        var tri: [Tri<V>]Int = 0
        for _ in 0..30:
            let (a, b, w) = (V(rand(1, 7)), V(rand(1, 7)), rand(1, 9))
            square[a, b] += w
            if a != b: square[b, a] += w
            tri[Tri<V>.sym(a, b)] += w
        for a in V:
            for b in V: assert tri[Tri<V>.sym(a, b)] == square[a, b]

@test
fn merging_stones_as_an_interval_dp():
    // the cheapest way to merge a row of piles, where a merge costs the size of the result
    for _ in 0..20:
        let n = rand(1, 8)
        domain I = 0..n
        let a = [I] _ => rand(1, 20)
        var dp: [Tri<I>]Int = 0
        for len in 2..=n:
            for t in Tri<I>.of_len(len):
                dp[t] = t.splits.map(((x, y)) => dp[x] + dp[y]).min()! + sum(a[i] for i in t.l..=t.r)
        assert dp[Tri<I>(I.first!, I.last!)] == merge_cost([a[i] for i in I])

/// The same answer by trying every merge of two neighbours.
fn merge_cost(piles: List<Int>) -> Int:
    if piles.len == 1: return 0
    var best = Int.MAX
    for i in 0..piles.len - 1:
        let merged = piles[i] + piles[i + 1]
        let rest = [piles[j] for j in 0..i] + [merged] + [piles[j] for j in i + 2..piles.len]
        best min= merged + merge_cost(rest)
    best
