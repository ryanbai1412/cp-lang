// Codeforces 2159 C: validate involution pairs among fixed entries of a[1..=n],
// then sum C(free, i) * invs[i + extra] over all subsets of free indices.
const MOD = 1_000_000_007
type Z = Mod<MOD>
let comb = Comb<Z>()

@!multitest
let n: Int = read()
var a: [0..=n]Int = read()

for i in 1..=n if a[i] != -1:
    if a[i] > n:
        print(0)
        return
    if a[i] != 0:
        if a[a[i]] == -1: a[a[i]] = i
        if a[a[i]] != i:
            print(0)
            return

var invs: [0..=n]Z = 1
for i in 2..=n: invs[i] = invs[i - 1] + invs[i - 2] * (i - 1)

let free = count(a[i] == -1 for i in 1..n)
let extra = Int(a[n] == -1)
print(sum(comb.choose(free, i) * invs[i + extra] for i in 0..=free))
