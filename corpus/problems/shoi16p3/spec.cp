// SDOI 2016 "Grade Comparison": n students, m courses with score caps and
// B's rank in each; count score tables where B beats exactly k students in
// every course. The score count of a course is a degree-n polynomial in its
// cap, so it is interpolated from its values at 0..=n + 1.
use std.poly.{lagrange_at}

type Z = Mod<1e9 + 7>
let (n, m, k) = read<(1..=100, 1..=100, 0..=99)>()
domain Course = [m]
let cap = read<[Course]1..=1e9>()
let rank = read<[Course]1..=n>()
let comb = Comb<Z>(n)

fn scores(c: Course) -> Z:
    let (above, below) = (n - rank[c], rank[c] - 1)
    let ys = [(1..=u).sum(of: s => Z(s) ** above * Z(u - s) ** below) for u in 0..=n + 1]
    lagrange_at(ys, Z(cap[c]))

var beaten = [0..n] j => Z(Int(j == k))
for c in Course.rev:
    let below = n - rank[c]
    let ways = scores(c)
    beaten = [0..n] j => ways * (0..=below).sum(of: t =>
        beaten[t] * comb.choose(j, t) * comb.choose(n - 1 - j, below - t)
    )
print(beaten[n - 1])
