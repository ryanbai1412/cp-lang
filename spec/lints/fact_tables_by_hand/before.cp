let (n, k): (1..=1000000, 0..=1000000) = read()
var fact: List<Int> = [1 for _ in 0..=n]
for i in 1..=n:
    fact[i] = fact[i - 1] * i % 998244353
print(fact[min(k, n)])
