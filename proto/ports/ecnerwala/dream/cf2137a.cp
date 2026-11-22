// Codeforces 2137 A — Collatz Conjecture
//
// Reverse `k` halving steps (`x -> 2 * x` each step) by printing `x << k`.
// With `k <= 20` and `x <= 20`, `x << k <= 20 * 2^20 < 2.1e7` cannot overflow.

@!multitest
let (k, x): (1..=20, 1..=20) = read()
print(x << k)
