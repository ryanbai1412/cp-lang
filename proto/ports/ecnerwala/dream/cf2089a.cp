// Codeforces 2089 A — Simple Permutation
//
// By Bertrand's postulate, for every `n >= 2` there exists a prime `p` with
// `n / 2 < p <= n`. Starting with `p` and interleaving pairs `(2p - k, k)` for
// `k = p + 1..=n` keeps every prefix ceiling-average equal to `p`:
// - After `2t + 1` elements, the sum is `(2t + 1) * p`, average `p`.
// - After `2t` elements, the sum is `2t * p - t`, whose ceiling average is
//   `ceil((2t * p - t) / (2t)) = p`.
// This yields `2(n - p) + 1 >= n / 3` prime prefix averages, and the remaining
// elements `1..=2p - n - 1` are appended at the end.

@!multitest
let n: 2..=100_000 = read()
let sieve = Sieve.new(n)
let p = first(x for x in n / 2 + 1..=n if sieve.is_prime(x))!

var res: List<Int> = [p]
for k in p + 1..=n:
    res.push(2 * p - k)
    res.push(k)
for rem in (1..=2 * p - n - 1).rev: res.push(rem)
print(res)
