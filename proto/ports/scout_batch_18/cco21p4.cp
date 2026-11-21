// CCO '21 P4 - Travelling Merchant (DMOJ: cco21p4, 25pt, TL 1.0s, ML 1024MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// Directed graph on n vertices and m roads. Road e: a -> b requires current
// money >= r and adds profit p >= 0. For each vertex v, find the minimum
// initial money to travel infinitely many roads, or -1 if impossible.
//
// Reverse peeling with sorted thresholds:
//   1) Peel all vertices whose out-degree reaches 0 (they cannot continue).
//   2) Among surviving edges, take the edge e = (u -> v) with the MAXIMUM
//      threshold r_e. Any walk starting at u with money r_e can traverse e
//      and then traverse ANY surviving edge forever (since all remaining
//      thresholds are <= r_e and profits >= 0). Record ans[u] min= r_e and
//      remove e; if out_deg[u] drops to 0, peel u in reverse, relaxing
//      predecessors w -> u via ans[w] min= max(r, ans[u] - p).
use std.graph.{Graph}

let (n, m): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
domain E = 0..m
let roads: [E](a: V, b: V, r: 0..=1e9, p: 0..=1e9) = read()

fn solve() -> [V]?Int:
    let rev = Graph<V, E>.directed([E] e => (u: roads[e].b, v: roads[e].a))
    var out_deg: [V]Int = 0
    for e in E: out_deg[roads[e].a] += 1

    var ans: [V]?Int = none
    var removed: [E]Bool = false
    var dead: [V]Bool = false
    var peel = [v for v in V if out_deg[v] == 0]
    var head = 0

    mut fn drain_peel():
        while head < peel.len:
            let v = peel[head]
            head += 1
            dead[v] = true
            for arc in rev.out(v):
                let e = arc.e
                if removed[e]: continue
                removed[e] = true
                let u = roads[e].a
                if let av = ans[v]:
                    let need = max(roads[e].r, av - roads[e].p)
                    ans[u] min= need
                out_deg[u] -= 1
                if out_deg[u] == 0 and not dead[u]:
                    peel.push(u)

    drain_peel()
    // WISH W14: `sorted(E, by: e => -roads[e].r)` allocates a List<E> and runs O(m log m)
    for e in sorted(E, by: e => -roads[e].r):
        if removed[e]: continue
        removed[e] = true
        let u = roads[e].a
        ans[u] min= roads[e].r
        out_deg[u] -= 1
        if out_deg[u] == 0:
            peel.push(u)
            drain_peel()
    ans

// Stress brute force: binary search initial money per vertex and check whether
// we can reach a positive-profit cycle or any directed cycle with current wealth.
@brute(solve)
fn solve_brute() -> [V]?Int
    where n <= 12 and m <= 25:
    fn can_infinite(start: V, money0: Int) -> Bool:
        // Bellman-Ford-style max-money relaxation over n + 1 steps:
        // if any vertex is still reachable after n edges (or reaches a cycle), infinite walk exists.
        var cur: [V]?Int = none
        cur[start] = money0
        for _ in 0..n:
            var nxt: [V]?Int = none
            for e in E:
                if let m0 = cur[roads[e].a]:
                    if m0 >= roads[e].r:
                        nxt[roads[e].b] max= m0 + roads[e].p
            cur = nxt
        any(cur[v] != none for v in V)

    var out: [V]?Int = none
    let hi_bound = max(roads[e].r for e in E)
    for v in V:
        if can_infinite(v, hi_bound):
            var (lo, hi) = (0, hi_bound)
            while lo < hi:
                let mid = (lo + hi) / 2
                if can_infinite(v, mid): hi = mid
                else: lo = mid + 1
            out[v] = lo
    out

let res = solve()
print(" ".join(str(res[v] ?? -1) for v in V))
