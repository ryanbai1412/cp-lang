let (a, b): (0..=1000, 0..=1000) = read()
var fact: List<Int> = [1]
for i in 1..=1000:
    fact.push(fact[i - 1] * i % 998244353)
fn inv(x: Int) -> Int = pow_mod(x, 998244353 - 2, 998244353)
if b > a:
    print(0)
else:
    print(fact[a] * inv(fact[b]) % 998244353 * inv(fact[a - b]) % 998244353)
