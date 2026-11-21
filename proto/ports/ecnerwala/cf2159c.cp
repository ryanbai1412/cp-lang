const MOD = 1_000_000_007
type Z = Mod<MOD>
let comb = Comb<MOD>()

@! multitest
let n: Int = read()
var a: [0..=n]Int = read()

for i in 1..=n if a[i] != -1:
    if a[i] > n:
        print(0)
        return
    if a[i] != 0:
        let target = a[i]
        if a[target] == -1: a[target] = i
        if a[target] != i:
            print(0)
            return

var invs: [0..=n]Z = 0
invs[0] = 1
for i in 1..=n:
    invs[i] = invs[i - 1]
    if i >= 2: invs[i] += invs[i - 2] * (i - 1)

let free = count(a[i] == -1 for i in 1..n)
let extra = Int(a[n] == -1)
print(sum(comb.choose(free, i) * invs[i + extra] for i in 0..=free))
