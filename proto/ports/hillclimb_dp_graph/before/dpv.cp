// AtCoder DP V, Subtree: for every vertex v of a tree, count the ways to
// paint some vertices black so that the black ones are connected and
// include v. Answers modulo m, which need not be prime.
let n: 1..=1e5 = read()
let m: 2..=1e9 = read()
domain V = 1..=n
domain E = 1..n

fn random_tree() -> [E](u: V, v: V):
    [E] e => (V(rand(1, e.label)), V(e.label + 1))

let edges: [E](u: V, v: V) = read(gen: random_tree)
var adj: [V]List<V> = []
for (u, v) in edges:
    adj[u].push(v)
    adj[v].push(u)
let tree = Rooted(adj, V.first)

// down[v]: connected black sets inside v's subtree that contain v
var down: [V]Int = 1
for v in tree.at.rev:
    if let p = tree.parent(v): down[p] = down[p] * (down[v] + 1) % m

// up[v]: 1 for leaving the parent's side empty, plus the connected black
// sets outside v's subtree that contain v's parent. No division mod m, so
// each child gets the product of its earlier and its later siblings.
var up: [V]Int = 1
for v in tree.at:
    var earlier = up[v]
    for c in adj[v]:
        if tree.parent(c) != v: continue
        up[c] = earlier
        earlier = earlier * (down[c] + 1) % m
    var later = 1
    for c in adj[v].rev:
        if tree.parent(c) != v: continue
        up[c] = (up[c] * later + 1) % m
        later = later * (down[c] + 1) % m

for v in V: print(down[v] * up[v] % m)
