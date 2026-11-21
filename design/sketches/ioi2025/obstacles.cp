// Cell (row i, column j) is free when T[i] > H[j]. Grow the free columns in
// increasing humidity (Kruskal over columns), each entering at the first row where
// the running max temperature beats it. When a column joins a neighbouring
// component, crossing between them needs a row as warm as the coldest row between
// their two entry rows, so it needs the component's first and last columns colder
// than that: those two columns are the join's needs. Two row-0 cells connect inside
// columns L..=R iff every join on their path in the join tree needs nothing right of
// R and nothing left of L: two path folds on a static tree.
@grader
mod obstacles:
    var joins: Tree<Col, (first: ?Col, last: ?Col)>   // filled by initialize

    fn initialize(t: [Row]0..=1e9, h: [Col]0..=1e9):
        let warmest = t.prefix_max()
        let entry: [Col]?Row = [Col] j => Row.first(where: i => warmest[i] > h[j])
        let coldest = RangeMin(t)
        let colder = SegTree(h, min)                   // first/last column below a temperature

        var parts = Dsu<Col>(payload: j => (span: j..=j, root: j))
        var edges: List<(Col, Col, (first: ?Col, last: ?Col))> = []
        for j in Col.sorted(by: h):
            for nb in [j - 1, j + 1] where nb in Col and parts.alive(nb):
                let p = parts.payload(nb)
                let need = if let (a, b) = (entry[p.root], entry[j])?: coldest[minmax(a, b)] else: 0
                edges.push((p.root, j, (first: colder.first(in: p.span, where: x => x < need),
                                        last: colder.last(in: p.span, where: x => x < need))))
                parts.union(nb, j, keep: (span: p.span ∪ parts.payload(j).span, root: j))
            parts.activate(j)
        joins = Tree.from_edges(edges, root: Col.max_by(h))

    fn can_reach(l: Col, r: Col, s: Col, d: Col) -> Bool:
        let path = joins.path_edges(s, d)              // binary lifting, O(log M)
        path.all(e => e.first? <= r ?? false) and path.all(e => e.last? >= l ?? false)
