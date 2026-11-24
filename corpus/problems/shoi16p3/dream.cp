// SHOI 2016 Unprecedented (score tables): in course c B ranks rank[c] with scores in 1..=top[c];
// exactly k others never beat B in any course. Count the score tables modulo 1e9+7.
// One course's tables are a polynomial in top of degree <= n: evaluate at 0..=n + 1 and
// interpolate. Then track how many others still have never been above B.
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
        )
        for j in Others
        if j >= keep
    )
print(still[Others.at(k)!])
