// CF 1900D Small GCD. Sorted, the element at j is the middle of n - 1 - j
// triples for each earlier i, contributing gcd(b_i, b_j). gcd(x, y) =
// sum over d | x, d | y of phi(d), so with cnt[d] = earlier multiples of d,
// sum_i gcd(b_i, b_j) = sum over d | b_j of phi(d) cnt[d]. The answer reaches
// C(8e4, 3) * 1e5 ~ 8.53e18, just under 2^63.
const MAXV = 100_000
let sv = Sieve(MAXV)
let phi = sv.phi_table()

@!multitest
let n: 3..=8e4 = read()
let a = read<[n]1..=1e5>()
let b = a.sorted()
var cnt = List.filled(MAXV + 1, 0)
var ans = 0
for (j, x) in b.items:
    var s = 0
    for d in sv.divisors(x): s += phi[d] * cnt[d]
    ans += s * (n - 1 - j)
    for d in sv.divisors(x): cnt[d] += 1
print(ans)
