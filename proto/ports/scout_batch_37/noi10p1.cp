// NOI 2010 Day 1 P1, Energy Collection
// original: corpus/submissions/5670455.cpp
let n: 1..=100_000 = read()
let m: 1..=100_000 = read()
let k = min(n, m)

// pairs[g]: plants (x, y) with gcd(x, y) == g; a plant with gcd g has g - 1
// plants between it and the machine and loses 2 * (g - 1) + 1
var pairs: [1..=k]Int = 0
var lost = 0
for g in (1..=k).rev:
    pairs[g] = (n / g) * (m / g) - sum(pairs[mult] for mult in (2 * g..=k).step(g))
    lost += pairs[g] * (2 * g - 1)
print(lost)
