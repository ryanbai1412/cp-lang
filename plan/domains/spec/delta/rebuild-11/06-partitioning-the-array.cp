// Codeforces 1920C: Partitioning the Array
// I/O: t tests; each n then a_1..a_n; print how many divisors k of n admit an m >= 2 making all n/k blocks equal mod m.
@!multitest
let n: 1..=2e5 = read()
let a = read<[n]1..=n>()
// blocks agree mod m iff m divides every a[i + k] - a[i]; some m >= 2 exists iff that gcd is not 1
fn good(k: Int) -> Bool:
    var g = 0
    for i in 0..(n - k): g = gcd(g, a[i + k] - a[i])
    g != 1
print(divisors(n).count(good))
