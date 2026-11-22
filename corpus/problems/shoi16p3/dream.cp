// SDOI 2016 "Grade Comparison": n students, m courses with score caps and
// B's rank in each; count score tables where B beats exactly k students in
// every course.
type Z = Mod<1e9 + 7>
let (n, m, k) = read<(1..=100, 1..=100, 0..=99)>()
domain Course = [m]
let cap = read<[Course]1..=1e9>()
let rank = read<[Course]1..=n>()
let comb = Comb<Z>(n)

fn scores(c: Course) -> Z =
    (1..=cap[c]).sum(of: s => Z(s).pow(n - rank[c]) * Z(cap[c] - s).pow(rank[c] - 1))

var beaten = [0..n] j => Z((j == k).int)
for c in Course.rev:
    let below = n - rank[c]
    beaten = [0..n] j => scores(c) * (0..=below).sum(of: t =>
        beaten[t] * comb.choose(j, t) * comb.choose(n - 1 - j, below - t))
print(beaten[n - 1])
