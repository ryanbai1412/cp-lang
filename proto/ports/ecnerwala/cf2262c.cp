// Codeforces 2262 C: the sorted values must interleave two arithmetic
// progressions with common difference d = M - M2: 0, d, 2d, ... (n/2 terms)
// and M, M - d, M - 2d, ... (the rest).  Try both candidates for the second
// largest value M2 and count the orders consistent with each.
const MOD = 1e9 + 7
type Z = Mod<MOD>
use std.seq.{dedup}
let comb = Comb<MOD>()
@!multitest
let n: 4..=2e5 = read()
let a: [0..n]Int = read()
let top = a[n - 1]
let half = n / 2
var seconds = sorted([a[n - 2], top - a[n - 2] / (half - 1)])
dedup(mut seconds)
var total = Z(0)
for second in seconds:
    let d = top - second
    var merged = [d * i for i in 0..half]
    for i in 0..n - half: merged.push(top - d * i)
    merged = sorted(merged)
    if all(merged[i] == a[i] for i in 0..n):
        total += comb.fact(n - half - 2) * comb.fact(half)
print(total)
