// Codeforces 2135 C — By the Assignment
//
// Assign each unfixed vertex (`none`, read from -1) a value in `0..lim` so
// every simple path between any two vertices has the same XOR of vertex values.
//
// Inside every 2-edge-connected component (bridge-cut component `C`), every
// vertex must share the same value; moreover, if a component contains an odd
// cycle (detected by an intra-component edge joining two vertices of equal
// BFS depth parity), that common value is forced to 0. Any component with no
// forced value (neither an odd cycle nor a pre-assigned vertex) contributes a
// factor of `lim` to the count; any conflict sets `ans = 0`.


type Z = Mod<998_244_353>

@!multitest
let (n, m, lim): (2..=200_000, 1..=400_000, 1..=10 ** 9) = read()
domain V = 1..=n
domain E = 0..m
let a: [V]?Int = read(none: -1)
let edges: [E](u: V, v: V) = read()
let g = Graph.undirected(edges)

let (domain C, comp) = two_edge_components(g)
let depth = bfs(g, [V(1)])

var want: [C]?Int = None
var ans: Z = 1
fn force(c: C, x: Int):
    if (want[c] ?? x) != x: ans = 0
    want[c] = x
for (u, v) in edges:
    if comp[u] == comp[v] and depth[u]! % 2 == depth[v]! % 2: force(comp[u], 0)
for u in V:
    if let x = a[u]: force(comp[u], x)
for c in C:
    if want[c] == None: ans *= Z(lim)
print(ans)
