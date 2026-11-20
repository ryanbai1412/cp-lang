// An Animal Contest 5 P2 (interactive): a hidden permutation of 1..n; asking
// "? i j" returns a[i] * a[j]. Find the permutation.
let n: 2..=1e5 = read()
domain I = 1..=n
domain Gap = 1..n                           // between positions g and g + 1

// the products of neighbours pin everything down once one value is known
var product: [Gap]Int = 0
for g in Gap:
    let p: 1..=n * n = ask("?", g, g.label + 1)
    product[g] = p

// the permutation that starts with `first`, if there is one
fn unfold(first: Int) -> ?[I]Int:
    var a: [I]Int = first
    var used: [I]Bool = false
    used[I(first)] = true
    for g in Gap:
        let here = a[I(g.label)]
        if product[g] % here != 0: return none
        let next = product[g] / here
        if next not in I or used[I(next)]: return none
        used[I(next)] = true
        a[I(g.label + 1)] = next
    a

for first in 1..=n:
    if product[Gap.first] % first == 0:
        if let a = unfold(first):
            print("!", a)
            break
