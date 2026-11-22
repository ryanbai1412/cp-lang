// Codeforces 2222 D — Prefix-Sum Descending Ranks
//
// Ordering positions `i in I` by ascending prefix sum `(pref.prefix(i), i)`
// and assigning descending ranks `n - rank[i]` ensures that every pair with
// `pref[i] < pref[j]` receives `p[i] > p[j]` (inverting the prefix-sum order).
// Inverting the permutation `ord` with `.inverse()` yields each index's rank directly.

@!multitest
let n: 1..=200_000 = read()
domain I = 0..n
let a: [I]-10 ** 9..=10 ** 9 = read()
let pref = prefix(a)
var ord: [I]I = [I] i => i
ord.sort(by: i => (pref.prefix(i.ord), i))
let rank = ord.inverse()
print([n - rank[i].ord for i in I])
