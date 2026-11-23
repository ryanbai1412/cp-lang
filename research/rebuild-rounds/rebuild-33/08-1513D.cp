// cf1513d GCD and MST: Kruskal by value; a[i] < p joins the run of multiples around i, and
// components are intervals, so a walk stops at the first non-multiple or joined neighbour.
@!multitest
let n: 2..=2e5 = read()
let p: 1..=1e9 = read()
domain V = 1..=n
let a = read<[V]1..=1e9>()
var dsu = Dsu<V>()
var total = 0
for i in V.sorted(by: i => a[i]):
    if a[i] >= p: break
    var j = i.prev
    while let u = j:
        if a[u] % a[i] != 0 or not dsu.unite(u, i): break
        total += a[i]
        j = u.prev
    j = i.next
    while let u = j:
        if a[u] % a[i] != 0 or not dsu.unite(u, i): break
        total += a[i]
        j = u.next
print(total + (dsu.parts - 1) * p)
