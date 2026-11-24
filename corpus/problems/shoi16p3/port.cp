// Source: SHOI 2016 / JLOI 2016 (成绩比较, Grade Comparison), https://dmoj.ca/problem/shoi16p3
//   (not public; statement read on Luogu as P3270)
// Problem: n <= 100 students including B, m <= 100 courses. Course i scores are 1..top[i]
//   (top <= 1e9) and B ranks rank[i] there (rank - 1 students strictly above him). Exactly k
//   other students score no higher than B in every course. Count the score tables modulo 1e9+7.
// Solution: per course, the scores alone count sum_{s=1..top} s^below (top - s)^above, a
//   polynomial in top of degree <= n: brute-force it at 0..=n + 1 and `lagrange_at` top.
//   Then still[j] = ways with j students still at or below B so far; a course keeps `keep` of
//   them at or below and fills its other below - keep slots from the n - 1 - j already out:
//   still'[keep] += still[j] C(j, keep) C(n - 1 - j, below - keep) ways. O(m n^2 log n).
// Bounds: n, m: 1..=100; k: 0..n; top: 1..=1e9; rank: 1..=n
// original: corpus/submissions/5445693.cpp
use std.poly.{lagrange_at}
type Z = Mod<1_000_000_007>
let (n, m): (1..=100, 1..=100) = read()
let k: 0..n = read()
domain Course = 0..m
let top = read<[Course]1..=1e9>()
let rank = read<[Course]1..=n>()
domain Others = 0..n
let comb = Comb<Z>(n + 2)

fn tables(c: Course) -> Z:
    let (above, below) = (rank[c] - 1, n - rank[c])
    lagrange_at(
        [sum(Z(s) ** below * Z(u - s) ** above for s in 1..=u) for u in 0..=n + 1],
        Z(top[c]),
    )

var still = [Others] j => Z(Int(j.label == n - 1))
for c in Course:
    let below = n - rank[c]
    let ways = tables(c)
    still = [Others] keep => ways * sum(
        still[j] * comb.choose(j.label, keep.label) * comb.choose(
            n - 1 - j.label,
            below - keep.label,
        ) for j in Others if j >= keep
    )
print(still[Others.at(k)!])
