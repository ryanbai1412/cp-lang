const MOD = 1_000_000_007
type Z = Mod<MOD>
let comb = Comb<MOD>()

@!multitest
let n: Int = read()
var a: [0..=n]Int = read()

var ok = true
for i in 1..=n:
    if a[i] != -1:
        if a[i] > n:
            ok = false
            break
        if a[i] != 0:
            let target = a[i]
            if a[target] == -1: a[target] = i
            if a[target] != i:
                ok = false
                break

if not ok:
    print(0)
else:
    var invs: [0..=n]Z = 0
    invs[0] = 1
    for i in 1..=n:
        invs[i] = invs[i - 1]
        if i >= 2: invs[i] += invs[i - 2] * (i - 1)
    var num_cnds = 0
    for i in 1..n:
        if a[i] == -1: num_cnds += 1
    let extra = Int(a[n] == -1)
    var ans: Z = 0
    for i in 0..=num_cnds:
        ans += comb.choose(num_cnds, i) * invs[i + extra]
    print(ans)
