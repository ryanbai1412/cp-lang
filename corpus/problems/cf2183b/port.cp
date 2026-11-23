// Codeforces 2183 B — Capped Array MEX
//
// Only `k - 1` elements survive at the end, so the final MEX can never exceed
// `k - 1`, and any value `< k - 1` absent from `a` can never be created.
// Hence the maximum achievable MEX is `min(MEX(a), k - 1)`, found as the
// smallest missing integer in `0..k - 1` (or `k - 1` if all appear).

@!multitest
let (n, k): (2..=200_000, 2..=200_000) = read()
let a: [0..n]0..=n = read()
var has: [0..k - 1]Bool = false
for x in a if x < k - 1: has[x] = true
print((0..k - 1).find(x => not has[x]) ?? (k - 1))
