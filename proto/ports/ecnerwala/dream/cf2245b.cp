// Codeforces 2245 B — Forced Half Selection
//
// At least `ceil(n / 2) = n - n / 2` elements must be taken, each contributing
// `a[i] - c`. Sorting `b = a.sorted()` with `mid = n / 2`, the largest `n - mid`
// elements `b[mid..n]` are always optimal to include as the mandatory half,
// while each element in the smaller half `b[0..mid]` is added iff `x - c > 0`.

@!multitest
let (n, c): (1..=200_000, -10 ** 9..=10 ** 9) = read()
let a: [0..n]-10 ** 9..=10 ** 9 = read()
let b = a.sorted()
let mid = n / 2
print(sum(max(x - c, 0) for x in b[0..mid]) + sum(x - c for x in b[mid..n]))
