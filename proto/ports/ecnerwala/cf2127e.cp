@!multitest
let (n, k): (Int, Int) = read()
domain V = 1..=n
let w: [V]Int = read()
let col_in: [V]Int = read()
var col: [V]?Int = [V] v => if col_in[v] == 0: none else: col_in[v] - 1

let edges: [0..n - 1](u: V, v: V) = read()
let tree = Rooted(Graph.undirected(edges), root: V(1))

var by_col: [0..k]List<V> = []
for v in V:
    if let c = col[v]: by_col[c].push(v)

var is_bad: [V]Bool = false
for c in 0..k:
    by_col[c].sort(by: v => tree.pos(v).ord)
    for (u, v) in by_col[c].adjacent:
        let l = tree.lca(u, v)
        if col[l] == none: col[l] = c
        elif col[l] != c: is_bad[l] = true

let ans = sum(w[v] for v in V if is_bad[v])
for v in tree.preorder if col[v] == none:
    col[v] = if let p = tree.parent(v): col[p] else: 0

print(ans)
print([col[v]! + 1 for v in V])
