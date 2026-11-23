// cf1548b Integers Have Friends: equal residues mod some m >= 2 iff the gcd of adjacent
// differences exceeds 1; the longest such window of differences, plus one.
@!multitest
let n: 1..=2e5 = read()
let a = read<[n]1..=1e18>()
let d = [abs(a[i + 1] - a[i]) for i in 0..n - 1]
let runs = d.maximal(state: Window(gcd), ok: w => (w.all ?? 0) != 1)
print(1 + (max(r.len for r in runs) ?? 0))
