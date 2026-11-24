// Source: Codeforces 2137B (Fun Permutation), https://codeforces.com/contest/2137/problem/B
// limits: 2000 ms, 256 MB
// Problem: a permutation p of 1..n, 2 <= n <= 2e5. Print any permutation q with
//   gcd(p_i + q_i, p_{i+1} + q_{i+1}) >= 3 for every adjacent pair. Multitest, sum n <= 2e5.
// Solution: q_i = n + 1 - p_i makes every p_i + q_i equal n + 1 >= 3, so every gcd is n + 1.
//   O(n).

@!multitest
let n: 2..=200_000 = read()
let p: [0..n]1..=n = read()
print([n + 1 - x for x in p])
