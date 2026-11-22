// Codeforces 2084 B — MIN = GCD
//
// Rearrange `a` so some prefix has `min == gcd` of the complementary suffix.
// The global minimum `mn = a[p]` must sit in the prefix, so the suffix must
// have `gcd == mn` — meaning every element in the suffix is a multiple of `mn`.
// Including more multiples of `mn` can only decrease the `gcd` toward `mn`, so
// a valid split exists iff the `gcd` of all multiples of `mn` other than `a[p]`
// equals `mn`.

@!multitest
let n: 2..=100_000 = read()
let a: [0..n]1..=10 ** 18 = read()
let p = argmin(a)!
let mn = a[p]
var g = 0
for i in 0..n if i != p and a[i] % mn == 0: g = gcd(g, a[i])
print(if g == mn: "Yes" else: "No")
