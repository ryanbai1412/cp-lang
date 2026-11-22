// Codeforces 2124 C — Subset Multiplication
//
// `b` was produced from a divisive array `a` (`a[i] | a[i + 1]`) by multiplying
// a subset of elements by some `k`. Whenever an adjacent pair `(x, y)` fails
// `y % x == 0`, `x` must have been multiplied by `k` while `y` was not, so
// `(x / k) | y`, which forces `(x / gcd(x, y)) | k`. Taking the LCM of
// `x / gcd(x, y)` across all non-dividing adjacent pairs yields a valid `k`.

@!multitest
let n: 2..=600_000 = read()
let b: [0..n]1..=10 ** 9 = read()
var res = 1
for (x, y) in b.adjacent if y % x != 0:
    res = lcm(res, x / gcd(x, y))
print(res)
