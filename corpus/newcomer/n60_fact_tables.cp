let (n, q): (1..=1000000, 1..=200000) = read()
var fact: List<Int> = [1 for _ in 0..=n]
for i in 1..=n:
    fact[i] = fact[i - 1] * i % 998244353  // expect: missing fact_tables_by_hand
fn inv(x: Int) -> Int:  // expect: hand_rolled_pow
    var b = x
    var e = 998244351
    var r = 1
    while e > 0:
        if e % 2 == 1:
            r = r * b % 998244353
        b = b * b % 998244353
        e /= 2
    return r
for _ in 0..q:
    let (a, b): (0..=n, 0..=n) = read()
    if b > a:
        print(0)
    else:
        print(fact[a] * inv(fact[b]) % 998244353 * inv(fact[a - b]) % 998244353)  // expect: binom_by_hand
