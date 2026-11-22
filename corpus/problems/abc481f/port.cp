// ABC481 F — Evil Shortest Path
// Directed weighted graph + O(N+M) layered lexicographic BFS + Mod<998244353> + @alt(program)

type Z = Mod<998_244_353>

let (n, m): (2..=2e5, 1..=2e5) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V, w: 1..=9) = read()

var adj: [V]List<(V, Int)> = []
for e in E:
    adj[edges[e].u].push((edges[e].v, edges[e].w))

var vis: [V]Bool = false
var ans: [V]Z = 0
vis[V.first] = true
var groups: List<List<V>> = [[V.first]]

while not groups.is_empty:
    var next_groups: List<List<V>> = []
    for grp in groups:
        let val = ans[grp[0]]
        var buckets: [0..10]List<V> = []
        for u in grp:
            for (to, c) in adj[u]:
                if not vis[to]:
                    buckets[c].push(to)
        for c in 1..=9:
            var ng: List<V> = []
            for to in buckets[c]:
                if not vis[to]:
                    vis[to] = true
                    ans[to] = val * 10 + c
                    ng.push(to)
            if not ng.is_empty:
                next_groups.push(ng)
    groups = next_groups

let out = [ans[v] for v in V if v != V.first]
print(out, sep: "\n")

@alt(program)
fn slow():
    var best_len: [V]?Int = none
    var best_str: [V]Str = ""
    best_len[V.first] = 0
    for step in 0..n:
        for e in E:
            let u = edges[e].u
            let v = edges[e].v
            let c = edges[e].w
            if let du = best_len[u]:
                let cand = best_str[u] + Str(['0' + c])
                let nd = du + 1
                if best_len[v] == none or nd < best_len[v]! or (nd == best_len[v]! and cand < best_str[v]):
                    best_len[v] = nd
                    best_str[v] = cand
    for v in V:
        if v != V.first:
            var acc: Z = 0
            for i in 0..best_str[v].len:
                acc = acc * 10 + (best_str[v][i] - '0')
            print(acc)
