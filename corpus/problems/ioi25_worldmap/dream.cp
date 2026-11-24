// Walk a DFS tree as an Euler tour (2n − 1 visits, consecutive visits joined by a
// tree edge) and paint one anti-diagonal of the 2n × 2n map per visit: neighbouring
// diagonals then touch only along tree edges. DFS leaves only edges to ancestors, so
// the first visit of v expands into three diagonals  v | ancestors joined to v | v,
// which places each back edge side by side. Diagonals shrink toward the far corner;
// touring the deepest root path last keeps every ancestor list inside its diagonal.
@grader
fn create_map(n: 1..=40, m: Int, a: List<1..=n>, b: List<1..=n>) -> List<List<1..=n>>:
    domain V = 1..=n
    domain E = 0..m
    let edges: [E](u: V, v: V) = [E] i => (u: V(a[i.ord]), v: V(b[i.ord]))
    let g = Graph.undirected(edges)
    let (tree, tree_edge) = g.dfs_tree(root: V.first!)
    let deep = tree.path(V.first!, V.max(by: v => tree.depth(v))!)
    var up: [V]List<V> = []  // up[v]: the ancestors v's back edges reach
    for e in g.edges().filter(e => not tree_edge[e.e]):
        let o = tree.orient(e.u, e.v)
        up[o.desc].push(o.anc)

    var diags: List<List<V>> = []
    var expanded = Bitset<V>.empty()
    for v in tree.euler_tour(by: c => deep.contains(c)):  // the deep child last
        diags.push([v])
        if up[v].len > 0 and not expanded.contains(v):
            expanded.add(v)
            diags.extend([up[v], [v]])
    diags.resize(4 * n - 1, diags.last!)

    let map = [2 * n] x => [2 * n] y =>
        let d = diags[x + y]
        (d.get(x - max(0, x + y - (2 * n - 1))) ?? d.last!).ord + 1
    return map.map(row => row.collect()).collect()
