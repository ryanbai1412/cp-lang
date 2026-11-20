// DMOPC '21 Contest 8 P5 — Balanced Split DP with Sparse Memoization (CP Lang v2 spec)
// original: corpus/submissions/4533533.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Minimize the cost to build an n-vertex structure from pieces of size 1..k-1
// with weight `v[i] = a[i + 1] + (i - 1) * a[1]`. Any optimal partition of `x`
// has a prefix sum in `[x/2 - k/2 - 1 .. x/2 + k/2 - 1]`, giving O(k^2 log n)
// distinct states. The C++ original hand-rolled a 10M dense cache + 8M open-
// addressed hash table (`namespace ht`); CP Lang v2 expresses it with `@memo`.

let (n, k): (2..=1e9, 2..=3000) = read()
domain K = 1..=k
let a: [K]Int = read()

let base = max(2 * k + 2, 4)
var v: [1..k]Int = 0
for i in 1..k:
    v[i] = a[K.from_idx(i)!] + (i - 1) * a[K.first]

var sm: [0..=base]?Int = none
sm[0] = 0
for i in 1..=base:
    for j in 1..=min(i, k - 1):
        if let prev = sm[i - j]:
            sm[i] min= prev + v[j]

// FRICTION UD5: `@memo` in v2 picks either flat (`<= 2^26`) or salted HashMap
// (`@memo`). For huge sparse integer DP (`n <= 1e9`), contests often use a
// hybrid small-prefix array + hash table to avoid hashing hot small keys; here
// the explicit `if x <= base: return sm[x]!` covers small keys manually, while
// `@memo` handles the O(k log n) large states.
@memo
fn dp(x: Int) -> Int:
    if x <= base: return sm[x]!
    let lo = x / 2 - k / 2 - 1
    min(dp(i) + dp(x - i) for i in lo..=(lo + k))!

print(dp(n - 2) + 2 * a[K.first])
