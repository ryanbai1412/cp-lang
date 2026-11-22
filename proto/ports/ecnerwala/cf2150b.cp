// Codeforces 2150 B — Grid Counting
//
// In an `n x n` grid, column `c in 0..n` can only be placed in rows
// `0..=min(c, n - 1 - c)`. Bucketing each column by its deepest allowed row
// `r = min(c, n - 1 - c)` gives `freq[r]` newly unlocked columns as we sweep
// `r` from `n - 1` down to `0`. Maintaining `cur` = unused unlocked columns,
// row `i` picks `a[i]` of them in `C(cur, a[i])` ways and consumes `a[i]`
// columns; valid iff `cur >= a[i]` at every row and `cur == 0` at the end.

const MOD = 998_244_353
type Z = Mod<MOD>
let comb = Comb<Z>()

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]0..=200_000 = read()
var freq: [0..n]Int = 0
for i in 0..n: freq[min(i, n - 1 - i)] += 1

var cur = 0
var ans: Z = 1
for i in (0..n).rev:
    cur += freq[i]
    if cur < a[i]:
        print(0)
        return
    ans *= comb.choose(cur, a[i])
    cur -= a[i]
print(if cur == 0: ans else: 0)
