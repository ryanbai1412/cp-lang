const MOD = 998244353
type Z = Mod<MOD>

@!multitest
let n: Int = read()
let a: [0..n]Int = read()
var freq: [0..n]Int = 0
for i in 0..n: freq[min(i, n - 1 - i)] += 1
var cur = 0
var numer: Z = 1
var denom: Z = 1
for i in (0..n).rev:
    cur += freq[i]
    if cur < a[i]:
        numer = 0
        break
    for z in 1..=a[i]:
        numer *= cur
        cur -= 1
        denom *= z
if cur != 0: numer = 0
print(numer / denom)
