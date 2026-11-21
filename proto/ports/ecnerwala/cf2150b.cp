const MOD = 998244353
type Z = Mod<MOD>
let comb = Comb<MOD>()

@!multitest
let n: Int = read()
let a: [0..n]Int = read()
var freq: [0..n]Int = 0
for i in 0..n: freq[min(i, n - 1 - i)] += 1

var cur = 0
var ans: Z = 1
for i in (0..n).rev:
    cur += freq[i]
    if cur < a[i]:
        print(0)
        return
    ans *= comb.choose(cur, a[i])
    cur -= a[i]
print(if cur == 0: ans else: 0)
