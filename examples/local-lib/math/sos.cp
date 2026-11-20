//! Sums over subsets and supersets of bitmasks, in O(n * 2^n) instead of O(3^n).

/// `out[mask]` is the sum of `f[sub]` over every `sub` that is a subset of `mask`.
/// `f` has `2 ** bits` entries.
pub fn sos_sub<T>(f: List<T>, bits: Int) -> List<T>:
    assert f.len == 1 << bits
    var g = [x for x in f]
    for b in 0..bits:
        for mask in 0..g.len:
            if (mask >> b) % 2 == 1: g[mask] = g[mask] + g[mask ^ (1 << b)]
    g

/// `out[mask]` is the sum of `f[sup]` over every `sup` that contains `mask`.
pub fn sos_sup<T>(f: List<T>, bits: Int) -> List<T>:
    assert f.len == 1 << bits
    var g = [x for x in f]
    for b in 0..bits:
        for mask in 0..g.len:
            if (mask >> b) % 2 == 0: g[mask] = g[mask] + g[mask | (1 << b)]
    g

/// The inverse of `sos_sub`: recover `f` from its subset sums.
pub fn sos_sub_inv<T>(g: List<T>, bits: Int) -> List<T>:
    assert g.len == 1 << bits
    var f = [x for x in g]
    for b in 0..bits:
        for mask in 0..f.len:
            if (mask >> b) % 2 == 1: f[mask] = f[mask] - f[mask ^ (1 << b)]
    f

@test
fn matches_enumerating_pairs():
    for bits in 0..=5:
        let n = 1 << bits
        let f = [rand(-9, 9) for _ in 0..n]
        let (down, up) = (sos_sub(f, bits), sos_sup(f, bits))
        for mask in 0..n:
            assert down[mask] == sum(f[s] for s in 0..n if s & mask == s)
            assert up[mask] == sum(f[s] for s in 0..n if s & mask == mask)
        assert sos_sub_inv(down, bits) == f

@test
fn count_pairs_with_disjoint_masks():
    // how many ordered pairs (i, j) have a[i] & a[j] == 0?
    let a = [1, 2, 4, 3, 5, 0]
    var cnt = List.filled(8, 0)
    for x in a: cnt[x] += 1
    let below = sos_sub(cnt, 3)
    assert sum(below[7 ^ x] for x in a) == count(x & y == 0 for x in a for y in a)
