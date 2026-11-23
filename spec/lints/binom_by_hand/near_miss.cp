let (a, b): (0..=1000, 0..=1000) = read()
var fact: List<Int> = [1]
for i in 1..=1000:
    fact.push(fact[i - 1] * i % 998244353)
print(fact[a] * fact[b] % 998244353)
