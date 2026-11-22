// Codeforces 2261 B — Adjacent Bitwise OR Raise
//
// An adjacent pair `(x, y)` that first differs at bit `d = bit_width(x ^ y) - 1`
// can set all bits `0..=d` on the larger side, reaching `x | ((1 << (d + 1)) - 1)`.
// Because `a[i] < 2^30`, `bit_width(x ^ y) <= 30`, so `1 << bit_width(...)`
// never overflows. When all elements are equal (`x == y` everywhere), no pair
// differs and the fallback `?? a[0]` returns the sole value.

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]0..=2 ** 30 - 1 = read()
print(max(x | ((1 << bit_width(x ^ y)) - 1) for (x, y) in a.adjacent if x != y) ?? a[0])
