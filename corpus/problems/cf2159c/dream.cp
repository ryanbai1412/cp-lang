// Codeforces 2159 C — Involution Completion Counting
//
// Each non-zero entry `a[i]` in `1..=n` must form an involution (`a[a[i]] == i`,
// and `a[i] <= n`); unknowns are read as `none` (from `-1`). Propagating all
// forced pairs leaves `free` unassigned indices in `1..n` (each of which may
// be set to 0 or join the involution) and `extra = Int(a[n] == none)` (index
// `n` cannot be 0).
//
// Choosing `i` of the `free` indices to be non-zero gives `choose(free, i)`
// choices, and the resulting `i + extra` non-zero indices can be paired into
// an involution in `invs[i + extra]` ways (`invs[k] = invs[k - 1] + (k - 1) * invs[k - 2]`).

const MOD = 1_000_000_007
type Z = Mod<MOD>
let comb = Comb<Z>()

@!multitest
let n: 1..=400_000 = read()
var a: [0..=n]?Int = read(none: -1)

for i in 1..=n if let v = a[i]:
    if v > n:
        print(0)
        return
    if v != 0:
        a[v] ??= i
        if a[v] != i:
            print(0)
            return

var invs: [0..=n]Z = 1
for i in 2..=n: invs[i] = invs[i - 1] + invs[i - 2] * (i - 1)

let free = count(a[i] == None for i in 1..n)
let extra = Int(a[n] == None)
print(sum(comb.choose(free, i) * invs[i + extra] for i in 0..=free))
