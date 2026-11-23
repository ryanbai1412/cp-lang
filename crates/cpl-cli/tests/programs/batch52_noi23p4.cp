// NOI '23 P4 (Trade): sum shortest paths over all reachable pairs in a complete binary
// tree of depth `logn` with upward parent edges and m directed shortcut edges, mod 998244353.
// DFS down the binary tree, snapshotting only the current subtree's distances before running
// subtree-restricted Dijkstra with Tropical Optional (?Int) distances and Mod<998244353>.
// original: corpus/submissions/5723063.cpp

const MOD = 998_244_353
type Z = Mod<MOD>

let (logn, m): (1..=18, 0..=200_000) = read()
let n = 1 << logn
domain V = 1..n

var w_par: [V]Int = 0
for v in 2..n: w_par[V(v)] = read()

var g: [V]List<(V, Int)> = []
for _ in 0..m:
    let (u, v, w): (V, V, Int) = read()
    g[u].push((v, w))

var dist: [V]?Int = None
var cache: [0..logn, V]?Int = None
var active = 0
var sum_dist: Z = 0
var ans: Z = 0

fn set_dist(u: V, val: ?Int):
    if let old = dist[u]:
        active -= 1
        sum_dist -= old
    dist[u] = val
    if let nw = val:
        active += 1
        sum_dist += nw

fn save_subtree(o: Int, d: Int):
    if o >= n: return
    cache[d, V(o)] = dist[V(o)]
    save_subtree(o * 2, d)
    save_subtree(o * 2 + 1, d)

fn restore_subtree(o: Int, d: Int):
    if o >= n: return
    set_dist(V(o), cache[d, V(o)])
    restore_subtree(o * 2, d)
    restore_subtree(o * 2 + 1, d)

fn dijkstra_subtree(root: Int, init_w: Int):
    var pq = MinHeap<(Int, V)>()
    fn relax(u_raw: Int, d: Int):
        if u_raw < root: return
        let u = V(u_raw)
        if let cur = dist[u] and cur <= d: return
        set_dist(u, d)
        pq.push((d, u))

    relax(root, init_w)
    while let (d, u) = pq.pop():
        if dist[u] != d: continue
        let ui = u.to_idx + 1
        if ui > 1: relax(ui / 2, d + w_par[u])
        for (v, w) in g[u]: relax(v.to_idx + 1, d + w)

fn dfs(o: Int, depth: Int, d_w: Int):
    save_subtree(o, depth)
    dijkstra_subtree(o, -d_w)
    ans += sum_dist + Z(d_w) * active
    if depth + 1 < logn:
        for c in [o * 2, o * 2 + 1]:
            dfs(c, depth + 1, d_w + w_par[V(c)])
    restore_subtree(o, depth)

dfs(1, 0, 0)
print(ans)
