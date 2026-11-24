// Source: NOI 2021 P1 (Light and Heavy Edges), https://dmoj.ca/problem/noi21p1
// Problem: a tree of n <= 1e5 vertices, every edge light. m <= 1e5 operations: "1 a b" makes
//   every edge with an endpoint on the a-b path light, then the path's edges heavy; "2 a b"
//   prints the number of heavy edges on the a-b path. Multitest, T <= 3.
// Solution: an edge is heavy iff both ends were last stamped by the same update (stamp 0 =
//   never), so an update stamps its path. On a heavy-light layout, a stamp tree holds each
//   vertex's stamp and a tied tree marks edges to the parent inside one chain range: an update
//   sets its ranges tied, then unties each range's first cell and the cell after it. A query sums
//   tied over its ranges and compares stamps across each range's top edge. O(m log^2 n).
// original: corpus/submissions/3859087.cpp
// Bounds: t: 1..=3
let t: 1..=3 = read()
for _ in 0..t:
    let n: 1..=1e5 = read()
    let m: 1..=1e5 = read()
    domain V = 1..=n
    domain E = 1..n

    fn random_tree() -> [E](u: V, v: V):
        [E] e => (V(rand(1, e.label)), V(e.label + 1))

    let edges: [E](u: V, v: V) = read(gen: random_tree)
    let tree = Rooted(Graph.undirected(edges), V.first!)

    // Stamp the vertices of each "1" path with the number of the command.
    // An edge is heavy when its two ends carry the same stamp, because any
    // later path through just one of them stamps it differently.
    // The tree is laid out so that a path is a few ranges of cells. Inside
    // a range, also keep one bit per cell: is the edge to the cell before
    // heavy? Only where two ranges of a path meet are stamps compared.
    var stamp = SegTree(+, [tree.Pre] _ => 0, act: assign)
    var tied = SegTree(+, [tree.Pre] _ => 0, act: assign)
    var who: [tree.Pre]V = V.first!
    for v in V: who[tree.sub(v).first!] = v

    for now in 1..=m:
        let kind: 1..=2 = read()
        let a: V = read()
        let b: V = read()
        if kind == 1:
            for chain in tree.chains(a, b):
                let c = chain.range
                let (l, r) = (c.first!, c.last!)
                stamp.apply(c, Some(now))
                tied.apply(c, Some(1))
                tied.set(l, 0)
                if r != tree.Pre.last: tied.set(tree.Pre.nth(r.ord + 1), 0)
        else:
            let peak = tree.sub(tree.lca(a, b)).first!
            var heavy = 0
            for chain in tree.chains(a, b):
                let c = chain.range
                let l = c.first!
                heavy += tied.fold(c) - tied[l]
                // every range but the highest hangs from the path by one more edge
                if l != peak:
                    let above = tree.sub(tree.parent(who[l])!).first!
                    if stamp[l] != 0 and stamp[l] == stamp[above]: heavy += 1
            print(heavy)
