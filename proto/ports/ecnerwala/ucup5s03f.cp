// Universal Cup 5 Stage 3 (Gorkovskaya) F.
const MOD = 998244353
type Z = Mod<MOD>

let (n, m): (1..=5000, 0..=5000) = read()
let a: [0..=m]0..=n = read()
var freq: [0..=n]Int = 0
for x in a: freq[x] += 1

let comb = Comb<Z>()
var ways = [comb.choose(n, i) for i in 0..=n]
var ans: Z = 0
for k in 0..=n:
    if k > 0:
        for i in 0..n: ways[i + 1] -= ways[i]
        for i in (0..n).rev: ways[i + 1] -= ways[i]
    ans += fold(*, ways[i] ** freq[i] for i in 0..=n if freq[i] > 0) * comb.choose(n, k)
print(ans / (Z(2) ** n))
