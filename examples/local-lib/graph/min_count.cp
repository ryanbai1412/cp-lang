//! Shortest-path distance + count semiring (`MinCount<P>`) and `floyd_warshall`.
use std.mod.{Mod}

/// A shortest-path distance paired with the number of shortest paths modulo `P`.
/// `dist == none` means unreachable (count 0).
pub struct MinCount<P>(dist: ?Int, ways: Mod<P>):
    /// Unreachable identity for `combine`.
    pub fn inf() -> MinCount<P> = MinCount(none, Mod<P>(0))

    /// Zero-length identity for `extend` (distance 0, 1 way).
    pub fn one() -> MinCount<P> = MinCount(0, Mod<P>(1))

    /// A single edge of weight `w`.
    pub fn edge(w: Int) -> MinCount<P> = MinCount(w, Mod<P>(1))

    /// Combine two alternative paths between the same endpoints (tropical `+`).
    pub fn combine(self, other: MinCount<P>) -> MinCount<P>:
        if dist == none: return other
        if other.dist == none: return self
        if dist! < other.dist!: return self
        if other.dist! < dist!: return other
        MinCount(dist, ways + other.ways)

    /// Concatenate two sequential paths `a -> k` and `k -> b` (tropical `*`).
    pub fn extend(self, other: MinCount<P>) -> MinCount<P>:
        if dist == none or other.dist == none: return MinCount<P>.inf()
        MinCount(dist! + other.dist!, ways * other.ways)

/// All-pairs shortest distances and shortest-path counts modulo `P` on a directed graph
/// with strictly positive edge weights (`w > 0`). O(n^3).
pub fn floyd_warshall<V, P>(edges: List<(V, V, Int)>) -> [V, V]MinCount<P>:
    var dp = [V, V] (_u, _v) => MinCount<P>.inf()
    for v in V: dp[v, v] = MinCount<P>.one()
    for (u, v, w) in edges:
        assert w > 0
        dp[u, v] = dp[u, v].combine(MinCount<P>.edge(w))
    for k in V:
        for i in V:
            if dp[i, k].dist == none: continue
            for j in V:
                if dp[k, j].dist == none: continue
                if i == k or k == j: continue
                dp[i, j] = dp[i, j].combine(dp[i, k].extend(dp[k, j]))
    dp

@test
fn floyd_warshall_min_count_matches_exhaustive_dag_and_positive_graphs():
    type Z = Mod<1000000007>
    domain V = 0..6
    // Hand-checked diamond + parallel edges: 0 -> 1 (2), 0 -> 2 (1), 1 -> 3 (1), 2 -> 3 (2), 0 -> 3 (3)
    // Shortest 0 -> 3 has length 3 with 3 distinct paths: (0-1-3), (0-2-3), (0-3)
    let g = [(V(0), V(1), 2), (V(0), V(2), 1), (V(1), V(3), 1), (V(2), V(3), 2), (V(0), V(3), 3)]
    let res = floyd_warshall::<V, 1000000007>(g)
    assert res[V(0), V(3)].dist == 3 and res[V(0), V(3)].ways == Z(3)
    assert res[V(0), V(0)].dist == 0 and res[V(0), V(0)].ways == Z(1)
    assert res[V(3), V(0)].dist == none and res[V(3), V(0)].ways == Z(0)

    // Differential test on random DAGs against DP path counting
    for _ in 0..15:
        var edges: List<(V, V, Int)> = []
        for i in 0..V.K:
            for j in i + 1..V.K:
                for _ in 0..rand(0, 2):
                    edges.push((V(i), V(j), rand(1, 5)))
        let fw = floyd_warshall::<V, 1000000007>(edges)
        for s in V:
            var best: [V]?Int = none
            var cnt = [V] _ => Z(0)
            best[s] = 0
            cnt[s] = Z(1)
            for i in s.to_idx..V.K:
                let u = V(i)
                if best[u] == none: continue
                for (a, b, w) in edges:
                    if a != u: continue
                    let nd = best[u]! + w
                    if best[b] == none or nd < best[b]!:
                        best[b] = nd
                        cnt[b] = cnt[u]
                    elif nd == best[b]!:
                        cnt[b] = cnt[b] + cnt[u]
            for t in V:
                assert fw[s, t].dist == best[t] and fw[s, t].ways == cnt[t]
