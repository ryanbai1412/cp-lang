// Walk a DFS tree as an Euler tour (2n − 1 visits, consecutive visits joined by a
// tree edge) and paint one anti-diagonal of the 2n × 2n map per visit: neighbouring
// diagonals then touch only along tree edges. DFS leaves only edges to ancestors, so
// the first visit of v expands into three diagonals  v | ancestors joined to v | v,
// which places each back edge side by side. Diagonals shrink toward the far corner;
// touring the deepest root path last keeps every ancestor list inside its diagonal.
@grader
fn create_map(n: 1..=40, m: Int, a: [m]1..=n, b: [m]1..=n) -> [2 * n][2 * n]1..=n:
    domain V = 1..=n
    let g = Graph<V>.undirected(zip(a, b))
    let tree = g.dfs_tree(root: V.first)
    let deep = tree.path(V.first, V.max_by(v => tree.depth[v]))
    let up = adjacency(g.edges.filter(e => not tree.has(e)).map(e => tree.orient_down(e).rev))

    var diags: List<List<V>> = []
    var expanded = Bitset<V>.empty
    for v in tree.euler_tour(order: c => c in deep):  // the deep child last
        diags.push([v])
        if up[v].len > 0 and expanded.insert(v): diags.extend([up[v], [v]])
    diags.pad_to(4 * n - 1, with: diags.last!)

    return [2 * n][2 * n] (x, y) =>
        let d = diags[x + y]
        d.get(x - max(0, x + y - (2 * n - 1))) ?? d.last!
