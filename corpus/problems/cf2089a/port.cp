// Source: Codeforces 2089A (Simple Permutation), https://codeforces.com/contest/2089/problem/A
// Problem: given n <= 1e5, print a permutation p of 1..n such that at least floor(n/3) - 1
//   of c_i = ceil((p_1 + ... + p_i) / i) are prime. Multitest (t <= 10).
// Solution: take a prime p with n/2 < p <= n (Bertrand), found with a sieve. Start with p,
//   then the pairs (2p - k, k) for k = p+1..=n: every prefix has ceiling average p, giving
//   2(n - p) + 1 >= n/3 primes. Append the unused 1..=2p-n-1 at the end. O(n).
// - After `2t + 1` elements, the sum is `(2t + 1) * p`, average `p`.
// - After `2t` elements, the sum is `2t * p - t`, whose ceiling average is
//   `ceil((2t * p - t) / (2t)) = p`.

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
