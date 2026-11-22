// Codeforces 2237 A — Prefix Minima Sum
//
// Print the sum of prefix minima of `a[0..n]`.

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]1..=10 ** 9 = read()
print(scan(a, min).sum())
