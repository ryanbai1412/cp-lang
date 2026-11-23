// Codeforces 2084 G2: Wish Upon a Satellite (Hard Version).
//
// Same reduction as G1: place values in increasing order; with c of the k
// smallest on even positions the beauty gains
//     gain(k, c) = (2c - k) * (2(n/2 - c) - (n - k)),
// and the answer is (max total gain + n(n + 1)^2 / 2) / 2.
//
// G1's best[c] is concave in c, and every step keeps it concave:
// - adding gain(k, ·), a quadratic with leading term -4c², lowers the j-th
//   slope by 8j plus a constant;
// - placing value k + 1 is a max-plus convolution: a fixed value shifts the
//   function by 0 (odd position) or 1 (even), a free one takes
//   max(best[c], best[c - 1]), which merges one 0 slope into the sorted
//   slopes.
// So keep best as a sorted multiset of slopes. O(n log n).
use std.slope.{Concave}

enum Side = Odd | Even

@!multitest
let n: 1..=500_000 = read()
domain P = 1..=n
let a: [P]?(1..=n) = read(none: 0)
let half = n / 2
let at = a.inverse()
fn side(p: P) -> Side = if p.label % 2 == 0: Even else: Odd
fn gain(k: Int, c: Int) -> Int = (2 * c - k) * (2 * (half - c) - (n - k))

var best = Concave.point(0)
for k in 0..n:
    best.add_quadratic(c => gain(k, c))
    best.max_convolve(match at[k + 1].map(side):
        Some(Odd) => Concave.point(0)
        Some(Even) => Concave.point(1)
        None => Concave.flat(0..=1))
print((best.at(half) + n * (n + 1) ** 2 / 2) / 2)
