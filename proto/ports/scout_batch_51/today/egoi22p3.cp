// Runnable today/ port of EGOI '22 P3 (Social Engineering) for proto/cpc.
// Stdin protocol: n, m, then m edges (u, v) in 1..=n.
// Prints "NO" if the strategy gives up, else "YES" followed by the number of paired paths
// and each path (len then 1-based vertices) so the checker verifies edge-disjointness for all games.
// original: corpus/submissions/5199301.cpp

fn main_solve():
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
        print("NO")
        return

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
        if special[u]: comp_special += 1
        for v in g[u]: build_tree(v, u)

    for u in g[root]:
        if vis[u]: continue
        roots.push(u)
        comp_special = 0
        build_tree(u, none)
        if comp_special % 2 == 1:
            print("NO")
            return

    var pairings: List<List<V>> = []

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
            var right = climb(open[i], u)
            right.rev()
            var full = climb(open[i + 1], u)
            full.push(u)
            for x in right: full.push(x)
            pairings.push(full)
            i += 2
        if open.len % 2 == 1: return open[open.len - 1]
        return none

    for r in roots:
        let _ = pair_sub(r)

    print("YES")
    print(pairings.len)
    for path in pairings:
        var line: List<Int> = [path.len]
        for v in path: line.push(v.idx + 1)
        print(line)

main_solve()
