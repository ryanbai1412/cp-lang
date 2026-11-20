// SAC '22 Code Challenge 3 P4: a tree whose edges get cut one at a time.
// Between cuts, report the distance from u to v, or -1 once they are apart.
let n: 2..=2e5 = read()
let q: 1..=2e5 = read()
domain V = 1..=n
domain E = 1..n

fn random_tree() -> [E](u: V, v: V):
    [E] e => (V(rand(1, e.label)), V(e.label + 1))

let edges: [E](u: V, v: V) = read(gen: random_tree)
let queries: [q](kind: 1..=2, u: V, v: V) = read()
var adj: [V]List<V> = []
for (u, v) in edges:
    adj[u].push(v)
    adj[v].push(u)
let tree = Rooted(adj, V.first)

// Backwards in time a cut is a join, so disjoint sets answer "still connected?".
var up: [V]?V = none
fn find(v: V) -> V:
    var r = v
    while let p = up[r]: r = p
    var x = v
    while let p = up[x]:
        up[x] = r
        x = p
    r

var answers: List<Int> = []
for (kind, u, v) in queries.rev:
    let (a, b) = (find(u), find(v))
    if kind == 2:
        if a != b: up[a] = b
    elif a == b: answers.push(tree.depth(u) + tree.depth(v) - 2 * tree.depth(tree.lca(u, v)))
    else: answers.push(-1)
for a in answers.rev: print(a)
