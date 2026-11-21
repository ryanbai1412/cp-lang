// Codeforces 2147 H: Maxflow GCD Coloring (dream).
// One color works iff the gcd of all Gomory-Hu tree cuts exceeds 1.
// Otherwise two colors suffice: mod 2, eliminate odd-degree vertices by
// complementing their alive neighborhood, then color back in reverse order.
with io(base: 1)

--- with multitest() ---

domain V = read(1..=50)
let m: 0..=1225 = read()
let edges: [m](u: V, v: V, w: 1..=1e6) = read()

let gh = FlowGraph<V>.undirected(edges).gomory_hu()
if gh.weights.fold(0, gcd) != 1:
    print(1)
    print(V.count)
    print(V)
    return

var adj: [V]Bitset<V> = Bitset.empty
for (u, v, w) in edges where w % 2 == 1:
    adj[u].toggle(v)
    adj[v].toggle(u)

fn flip_clique(nbr: Bitset<V>):
    for u in nbr:
        adj[u] ^= nbr
        adj[u].remove(u)

var alive = Bitset<V>.full
var stk: List<V> = []
while let v = alive.find(v => (adj[v] & alive).len % 2 == 1):
    alive.remove(v)
    stk.push(v)
    flip_clique(adj[v] & alive)

var side1 = Bitset<V>.empty
while let v = stk.pop():
    let nbr = adj[v] & alive
    flip_clique(nbr)
    alive.add(v)
    if (nbr & side1).len % 2 == 0: side1.add(v)

let (c1, c0) = V.partition(v => v in side1)
print(2)
print(c0.len)
print(c0)
print(c1.len)
print(c1)
