// CF 1923E Count Paths: walk the tree keeping, per colour, how many visited vertices are still
// visible (no same-coloured vertex between them and the walk's position).
@!multitest
let n: 2..=2e5 = read()
domain V = 1..=n
let col = read<[V]1..=n>()
let t = Rooted(Graph.undirected(read<[n - 1](u: V, v: V)>()), root: V.first!)
var seen: [1..=n]Int = 0
var before: [V]Int = 0
var paths = 0
for step in t.tour:
    match step:
        Enter(v, parent) =>
            if let p = parent: seen[col[p]] = 1
            before[v] = seen[col[v]]
            paths += before[v]
        Exit(v, _) =>
            seen[col[v]] = before[v] + 1
        Again(_, _) => ()
print(paths)
