// Runnable port (proto/cp.py lib-run) of etsr23r4p3.cp
// original: corpus/submissions/5635626.cpp
use std.graph.{Graph}
use std.tree.{Rooted}

let (n, m): (Int, Int) = read()
domain V = 1..=n
domain TE = 0..(n - 1)

var has_edge: [V][V]Bool = [V] _ => [V] _ => false
for _ in 0..m:
    let (u, v): (Int, Int) = read()
    has_edge[V(u)][V(v)] = true
    has_edge[V(v)][V(u)] = true

var tu: List<V> = []
var tv: List<V> = []
for _ in 0..(n - 1):
    let (u, v): (Int, Int) = read()
    tu.push(V(u))
    tv.push(V(v))

let g = Graph<V, TE>.undirected([TE] e => (u: tu[e.to_idx], v: tv[e.to_idx]))
let tree = Rooted<V>(g, root: V.first)
let post = tree.postorder

var ans = 0
for mask in 1..(1 << n):
    var allowed: List<V> = []
    for i in 0..n:
        if ((mask >> i) & 1) == 1:
            allowed.push(V(i + 1))

    var dp: [V][V]Int = [V] _ => [V] _ => 0
    for u in post:
        for v1 in allowed:
            var ways = 1
            for c in tree.children(u):
                var cmul = 0
                for v2 in allowed:
                    if has_edge[v1][v2]:
                        cmul += dp[c][v2]
                ways *= cmul
            dp[u][v1] = ways

    var total = 0
    for v in allowed:
        total += dp[V.first][v]

    if ((n - allowed.len) & 1) == 0:
        ans += total
    else:
        ans -= total

print(ans)
