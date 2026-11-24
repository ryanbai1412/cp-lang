// Source: Codeforces 2150B (Grid Counting), https://codeforces.com/contest/2150/problem/B
// limits: 2000 ms, 256 MB
// Problem: colour cells of an n x n grid (n <= 2e5) black so that row k has a_k black cells and,
//   for each k, exactly one black cell has max(x, y) = k and exactly one has max(x, n+1-y) = k.
//   Count the grids mod 998244353. Multitest, sum n <= 2e5.
// Solution: the conditions mean one black cell per column y, in a row x <= min(y, n+1-y).
//   Going up from the bottom row, row i (0-based) unlocks the freq[i] columns whose deepest
//   allowed row is i and picks its a_i cells among the cur unused unlocked ones: C(cur, a_i)
//   ways. The answer is the product, or 0 if some cur < a_i or columns are left. O(n).

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
