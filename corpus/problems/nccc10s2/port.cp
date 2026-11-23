// cp: 0.1 8ef64b9+dirty
let n: 1..=1000 = read()
let k: 1..=26 = read()
domain L = 'A'..'A' + k
let orders: [n]Perm<L> = read()

var ahead: [L, L]Bool = true  // ahead[x, y]: x before y in every order
for o in orders:
    for (x, y) in o.pairs():
        ahead[y, x] = false

@memo
fn chain(u: L) -> Int:
    ([chain(v) for v in L if v != u and ahead[u, v]].max()! ?? 0) + 1

print([chain(u) for u in L].max()!)
