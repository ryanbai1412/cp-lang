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
let tree = Rooted(Graph.undirected(edges), V.first!)

// Backwards in time a cut is a join, so one Dsu answers "still connected?".
var joined = Dsu<V>()
var answers: List<Int> = []
for (kind, u, v) in queries.rev:
    if kind == 2: joined.unite(u, v)
    elif joined.same(u, v): answers.push(
        tree.depth(u) + tree.depth(v) - 2 * tree.depth(tree.lca(u, v))
    )
    else: answers.push(-1)
for a in answers.rev: print(a)
