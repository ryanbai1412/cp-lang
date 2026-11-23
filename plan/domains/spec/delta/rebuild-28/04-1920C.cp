// CF 1920C Partitioning the Array: k scores iff the gcd of all |a[i+k] - a[i]| is not 1.
@!multitest
let n: 1..=2e5 = read()
let a = read<[n]1..=n>()

fn scores(k: Int) -> Bool:
    var g = 0
    for i in 0..n - k:
        g = gcd(g, abs(a[i + k] - a[i]))
    g != 1

print(divisors(n).count(scores))
