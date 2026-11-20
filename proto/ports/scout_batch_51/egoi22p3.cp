// EGOI '22 P3 (Social Engineering): interactive edge-disjoint return game on a graph.
// Mark all neighbours of vertex 1 as special; decompose G \ {1} into spanning trees.
// If every component has an even count of special vertices, pair them bottom-up into
// edge-disjoint tree paths so each opponent move from 1 has a precomputed path back.
// original: corpus/submissions/5199301.cpp

@grader fn GetMove() -> Int
@grader fn MakeMove(v: Int)

@grader
fn SocialEngineering(n: Int, m: Int, edges: List<(Int, Int)>):
    domain V = 1..=n
    let root = V.first
    let g = Graph.undirected([(V.from_idx(u - 1)!, V.from_idx(v - 1)!) for (u, v) in edges])
    if g[root].len % 2 == 1: return

    var special: [V]Bool = false
    for u in g[root]: special[u] = true

    var vis: [V]Bool = false
    var par: [V]?V = none
    var kids: [V]List<V> = []
    var roots: List<V> = []
    var comp_special = 0

    fn build_tree(u: V, p: ?V):
        if u == root or vis[u]: return
        vis[u] = true
        par[u] = p
        if let parent = p: kids[parent].push(u)
        comp_special += Int(special[u])
        for v in g[u]: build_tree(v, u)

    for u in g[root] if not vis[u]:
        roots.push(u)
        comp_special = 0
        build_tree(u, none)
        if comp_special % 2 == 1: return

    var pair_path: [V]List<V> = []

    fn climb(mut v: V, top: V) -> List<V>:
        var up: List<V> = []
        while v != top:
            up.push(v)
            v = par[v]!
        up

    fn pair_sub(u: V) -> ?V:
        var open: List<V> = []
        if special[u]: open.push(u)
        for v in kids[u]:
            if let carry = pair_sub(v): open.push(carry)
        for i in (0..open.len - 1).step(2):
            var right = climb(open[i], u)
            reverse(mut right)
            let full = climb(open[i + 1], u) + [u] + right
            let rev_full = [x for x in full.rev]
            pair_path[full.first!] = full
            pair_path[full.last!] = rev_full
        if open.len % 2 == 1: open.last else: none

    for r in roots: pair_sub(r)

    while true:
        let raw = GetMove()
        if raw == 0: return
        let u = V.from_idx(raw - 1)!
        for v in pair_path[u][1..]: MakeMove(v.to_idx + 1)
        MakeMove(1)
