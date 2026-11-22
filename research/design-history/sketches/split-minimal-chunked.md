# Split the Attractions: minimal stdlib, decomposed

Same stdlib budget as `split-minimal.md`. Two generic helpers sit at the top level (what a personal library
would hold); the problem's steps are nested chunks.

```cp
fn adjacency<D>(edges: List<(D, D)>) -> [D]List<D>:
    var adj: [D]List<D> = []
    for (u, v) in edges:
        adj[u].push(v)
        adj[v].push(u)
    return adj

// BFS forest: one tree per unseen root, each tree contiguous in `order`.
fn bfs<D>(adj: [D]List<D>, roots: List<D>, allow: (D) -> Bool = _ => true) -> (List<D>, [D]?D):
    var parent: [D]?D = none, seen: [D]Bool = false
    var order: List<D> = []
    var i = 0
    for r in roots if not seen[r]:
        seen[r] = true
        order.push(r)
        while i < order.len:
            for w in adj[order[i]] if allow(w) and not seen[w]:
                seen[w] = true
                parent[w] = order[i]
                order.push(w)
            i += 1
    return (order, parent)

domain Group = labels(1..=3)

@grader
fn find_split(n: I32, a: I32, b: I32, c: I32, p: List<I32>, q: List<I32>) -> List<I32>:
    domain V = n
    let N = V.count
    let edges = zip(p, q).map((x, y) => (V.at(x), V.at(y))).list()
    let adj = adjacency(edges)
    let want: [Group]Int = [a, b, c]
    let [small, mid, big] = Group.sorted(by: want)
    let (s, m) = (want[small], want[mid])
    let (order, parent) = bfs(adj, roots: [V.first])

    fn centroid() -> V:
        var size: [V]Int = 1, heavy: [V]Int = 0
        for v in order.rev():
            if let u = parent[v]:
                size[u] += size[v]
                heavy[u] max= size[v]
        return V.first(where: v => max(heavy[v], N - size[v]) <= N / 2)!

    fn pieces(cut: V) -> (domain Piece, [V]?Piece):
        let (domain Piece, roots) = order.filter(v => v != cut and parent[v] in [none, cut]).numbered()
        var of: [V]?Piece = none
        for x in Piece: of[roots[x]] = x
        for v in order if v != cut and of[v] == none: of[v] = of[parent[v]!]
        return (Piece, of)

    let centre = centroid()
    let (domain Piece, of) = pieces(cut: centre)
    var psize: [Piece]Int = 0
    for v in V:
        if let x = of[v]: psize[x] += 1

    fn join() -> ?List<Piece>:                        // a connected run of pieces with total in [s, 2s)
        var padj: [Piece]List<Piece> = []
        for (u, v) in edges:
            if let x = of[u], y = of[v], x != y:
                padj[x].push(y)
                padj[y].push(x)
        let (porder, pparent) = bfs(padj, roots: Piece.list())
        var run: List<Piece> = [], total = 0
        for x in porder:
            if pparent[x] == none:
                (run, total) = ([], 0)
            run.push(x)
            total += psize[x]
            if total >= s: return run
        return none

    fn take(k: Int, inside: [V]Bool) -> List<V>:     // a BFS prefix is connected; traps if fewer than k
        let (got, _) = bfs(adj, roots: [V.first(where: inside)!], allow: inside)
        return got[.. k].list()

    let chosen = Piece.first(where: x => psize[x] >= s).map(x => [x]) ?? join() ?? return List.filled(n, 0)
    var picked: [Piece]Bool = false
    for x in chosen: picked[x] = true
    let region = V.map(v => picked[of[v]?] ?? false)

    var label: [V]Group = big
    for v in take(s, inside: region): label[v] = small
    for v in take(m, inside: V.map(v => not region[v])): label[v] = mid
    return label.map(x => I32(x.label)).list()
```
