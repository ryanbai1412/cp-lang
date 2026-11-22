// Codeforces 2137 B — Fun Permutation
//
// Setting `q[i] = n + 1 - p[i]` gives a valid permutation of `1..=n` in which
// every position has `p[i] + q[i] == n + 1`. Thus every adjacent pair satisfies
// `gcd(p[i] + q[i], p[i + 1] + q[i + 1]) == n + 1 >= 3` (since `n >= 2`).

@!multitest
let n: 2..=200_000 = read()
let p: [0..n]1..=n = read()
print([n + 1 - x for x in p])
