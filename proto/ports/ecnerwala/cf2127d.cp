const MOD = 1_000_000_007
type Z = Mod<MOD>
let comb = Comb<MOD>()

@!multitest
let (n, m): (Int, Int) = read()
domain V = 1..=n
let edges: [0..m](u: V, v: V) = read()
if m > n - 1:
    print(0)
    return

var deg: [V]Int = 0
for (u, v) in edges:
    deg[u] += 1
    deg[v] += 1

var deg2: [V]Int = 0
var num_e = 0
for (u, v) in edges if deg[u] > 1 and deg[v] > 1:
    deg2[u] += 1
    deg2[v] += 1
    num_e += 1

if any(deg2[v] >= 3 for v in V):
    print(0)
    return

if num_e >= 1:
    var ans: Z = 4
    for v in V if deg[v] > 1: ans *= comb.fact(deg[v] - deg2[v])
    print(ans)
else:
    print(Z(2) * comb.fact(n - 1))
