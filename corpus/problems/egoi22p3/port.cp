// Source: EGOI 2022 Day 1 Social Engineering, https://dmoj.ca/problem/egoi22p3
// Problem: interactive: a connected graph, n <= 2e5, m <= 4e5, Maria at vertex 1. Moves walk
//   along unused edges, Maria choosing hers and you everyone else's; whoever cannot move loses.
//   Give up at once if Maria wins, else play until she has no move.
// Solution: Maria loses iff her neighbours pair up by edge-disjoint paths avoiding her: then each
//   move into a neighbour goes along its path to its partner and back to her. That needs an even
//   degree and an even number of neighbours in each component of the graph without her. DFS
//   trees per component; bottom up each vertex pairs its open neighbours through itself and
//   passes at most one up. O(n + m).
// Stdin protocol (the grader played over stdin, as in spec.cp): n, m, then m edges (u, v) in
//   1..=n. Prints "!" at once when the strategy gives up. Otherwise asks "?" for Maria's move (0
//   once she resigns), answers a move into a neighbour along its paired path and back to vertex
//   1, and prints "!" once she has resigned.
// original: corpus/submissions/5199301.cpp

fn main_solve():
    let wire = Wire()
    let n: Int = read()
    let m: Int = read()
    domain V = 0..n
    var g: [V]List<V> = []
    for _ in 0..m:
        let u: Int = read()
        let v: Int = read()
        g[V(u - 1)].push(V(v - 1))
        g[V(v - 1)].push(V(u - 1))

    let root = V(0)
    if g[root].len % 2 == 1:
        print("!")
        return

    var special: [V]Bool = false
    for u in g[root]: special[u] = true

    var vis: [V]Bool = false
    var par: [V]?V = None
    var kids: [V]List<V> = []
    var roots: List<V> = []
    var comp_special = 0

    fn build_tree(u: V, p: ?V):
        if u == root or vis[u]: return
        vis[u] = true
        par[u] = p
        if let parent = p: kids[parent].push(u)
        if special[u]: comp_special += 1
        for v in g[u]: build_tree(v, u)

    for u in g[root]:
        if vis[u]: continue
        roots.push(u)
        comp_special = 0
        build_tree(u, None)
        if comp_special % 2 == 1:
            print("!")
            return

    // Each paired path from either end: to its partner, then back to Maria's vertex.
    var route: [V]List<V> = []

    fn climb(start: V, top: V) -> List<V>:
        var up: List<V> = []
        var cur = start
        while cur != top:
            up.push(cur)
            cur = par[cur]!
        return up

    fn pair_sub(u: V) -> ?V:
        var open: List<V> = []
        if special[u]: open.push(u)
        for v in kids[u]:
            if let carry = pair_sub(v): open.push(carry)
        var i = 0
        while i + 1 < open.len:
            let right = climb(open[i], u)
            var full = climb(open[i + 1], u)
            full.push(u)
            for x in right.rev: full.push(x)
            route[full.last!] = full.rev.to_list()
            route[full.first!] = full
            i += 2
        if open.len % 2 == 1: return open.last!
        return None

    for r in roots:
        let _ = pair_sub(r)

    while true:
        let v: Int = wire.ask("?")
        if v == 0: break
        for u in route[V(v - 1)].drop(1): print(u.idx + 1)
        print(1)
    print("!")

main_solve()
