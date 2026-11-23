// CF 1995B1 Bouquet (Easy Version)
// in: t; per test n m, a_1..a_n. out: the most petals with spread at most 1 and cost at most m.
@!multitest
let (n, m): (1..=2e5, 1..=1e18) = read()
var a = read<[n]1..=1e9>()
a.sort()
var best = 0
var total = 0
var l = 0
for r in 0..n:
    total += a[r]
    while a[r] - a[l] > 1 or total > m:
        total -= a[l]
        l += 1
    best max= total
print(best)
