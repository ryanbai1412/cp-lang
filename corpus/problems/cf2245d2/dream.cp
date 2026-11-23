// Codeforces 2245 D2: assign each vertex a non-zero integer so every kind-1
// edge has a[u] + a[v] > 0 and every kind-2 edge has a[u] + a[v] < 0.
//
// Peel any vertex whose remaining incident edges all have the same kind (at
// least one kind count is 0). Give the z-th peeled vertex magnitude n - z,
// positive if its remaining edges are kind 1 (cnt[v, 2] == 0) and negative if
// they are kind 2 (cnt[v, 1] == 0). Since later vertices have strictly smaller
// magnitude, each vertex's sign decides every edge incident to it at peel time.
// If peeling stalls before all n vertices are removed, no assignment exists.
@!multitest
let (n, m): (1..=2e5, 0..=2e5) = read()
domain V = 1..=n
domain E = 0..m
domain Kind = 1..=2
let raw: [E](kind: Kind, u: V, v: V) = read()
let g = Graph.undirected([E] e => (u: raw[e].u, v: raw[e].v))

fn solve() -> ?[V]Int:
    var cnt: [V, Kind]Int = 0
    for e in E:
        cnt[raw[e].u, raw[e].kind] += 1
        cnt[raw[e].v, raw[e].kind] += 1

    fn drop_edge(v: V, a: (to: V, e: E)):
        cnt[a.to, raw[a.e].kind] -= 1

    var ans: [V]Int = 0
    let order = g.peel(
        ready: v => cnt[v, Kind(1)] == 0 or cnt[v, Kind(2)] == 0,
        on_remove: drop_edge,
    )
    if order.len < n: return None
    for (z, v) in order.items:
        let mag = n - z
        ans[v] = if cnt[v, Kind(1)] == 0: -mag else: mag
    ans

if let ans = solve():
    print("YES")
    print(ans)
else:
    print("NO")
