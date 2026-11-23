// Universal Cup 5 Stage 2 F — Periodic Intervals
//
// For a query period t, we seek the smallest multiple 6 * k <= t (k >= 1) such
// that every grid point (6 * i + 6) * t / (6 * k) (i in 0..k) falls in some
// sorted interval [l, r] of p0, and for every residue v in 1..=5 every point
// (6 * i + v) * t / (6 * k) falls in some interval of p1.
//
// Clearing denominators by 6 * k (using I128 since k * r can exceed 64 bits):
//   l <= (6 * i + v) * t / (6 * k) <= r
//   <=> 6 * k * l - t * v <= 6 * t * i <= 6 * k * r - t * v.
// Scanning the sorted intervals advances the first uncovered index i greedily.
//
// Whenever residue v fails at k, any valid multiple k' = m * k must refine the
// 6-grid enough that the failing point leaves residue class v mod 6, which
// requires m to be a multiple of 6 / gcd(6, v):
// - v in {1, 5} (gcd = 1) -> multiply k by 6;
// - v in {2, 4} (gcd = 2) -> multiply k by 3;
// - v = 3       (gcd = 3) -> multiply k by 2.
// If residue 6 (p0) fails, no multiple of k can fix it (return none).

fn covers(k: I128, v: I128, segs: List<(Int, Int)>, t: I128) -> Bool:
    let k6 = I128(6) * k
    let t6 = I128(6) * t
    let shift = t * v
    var i: I128 = 0
    for (l, r) in segs:
        if i >= k: return true
        let lo = k6 * I128(l) - shift
        let hi = k6 * I128(r) - shift
        if lo > t6 * i: return false
        if hi >= t6 * i: i = hi / t6 + 1
    i >= k

fn solve(t: I128, p0: List<(Int, Int)>, p1: List<(Int, Int)>) -> ?Int:
    var k: I128 = 1
    while I128(6) * k <= t:
        if not covers(k, 6, p0, t): return None
        if not covers(k, 1, p1, t) or not covers(k, 5, p1, t): k *= 6
        elif not covers(k, 2, p1, t) or not covers(k, 4, p1, t): k *= 3
        elif not covers(k, 3, p1, t): k *= 2
        else: return Some(Int(I128(6) * k))
    None

let (n0, n1): (1..=200_000, 1..=200_000) = read()
let p0: [0..n0](Int, Int) = read()
let p1: [0..n1](Int, Int) = read()
let q: 1..=200_000 = read()
let queries: [0..q]1..=10 ** 18 = read()
for t in queries:
    print(solve(I128(t), p0, p1) ?? -1)
