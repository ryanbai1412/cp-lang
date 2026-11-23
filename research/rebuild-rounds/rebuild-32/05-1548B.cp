// cf1548b Integers Have Friends: a friend group is a run whose adjacent differences
// have gcd >= 2; the longest such run of differences, plus one.
use std.seq.{maximal}

@!multitest
let n: 1..=2e5 = read()
let a = read<[n]1..=1e18>()
let d = [0..n - 1] i => abs(a[i + 1] - a[i])
let runs = d.maximal(state: Window(gcd), ok: w => (w.all ?? 0) != 1)
print((max(r.len for r in runs) ?? 0) + 1)
