// Source: NOI 2021 P3 (Celebration), https://dmoj.ca/problem/noi21p3
// Problem: a weakly connected digraph, n, q <= 3e5, m <= 6e5, where x => z and y => z imply
//   x => y or y => x. Each query gives s, t and k <= 2 extra edges usable only by it; print
//   how many cities lie on some s-to-t walk (0 if t is unreachable).
// Solution: condense SCCs, weighted by size. The property makes each component's
//   predecessors a chain, so the condensed DAG's transitive reduction is an out-tree: Kahn's
//   peel hangs each component under the predecessor that frees it last. Per query, take the
//   virtual tree of the <= 6 key components plus the extra edges, BFS forward from s and
//   backward from t, and count each component (or inner path of a tree edge) both reach.
//   O(n + m) setup, O(k log n) per query.

let (n, m, q, kv): (Int, Int, Int, Int) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V) = read()
let g = Graph<V, E>.directed(edges)

// 1. Strongly connected components and component weights.
let (C, comp) = scc<V>(u => g.out(u))
var wt: [C]Int = 0
for u in V: wt[comp[u]] += 1

// 2. Peel the unique spanning tree of the condensed DAG.
var dag_adj: [C]List<C> = []
var indeg: [C]Int = 0
for e in E:
    let (cu, cv) = (comp[edges[e].u], comp[edges[e].v])
    if cu != cv:
        dag_adj[cu].push(cv)
        indeg[cv] += 1

var root = C.first!
for c in C:
    if indeg[c] == 0: root = c

var tree_adj: [C]List<C> = []
var stack = [root]
while not stack.is_empty:
    let u = stack.pop()!
    for v in dag_adj[u]:
        indeg[v] -= 1
        if indeg[v] == 0:
            tree_adj[u].push(v)
            stack.push(v)

let tree = Rooted(u => [(to: v, w: wt[v]) for v in tree_adj[u]], root)

// 3. Per-query virtual tree + forward/backward reachability.
var fwd: [C]List<C> = []
var rev: [C]List<C> = []
var vis_f: [C]Bool = false
var vis_r: [C]Bool = false

for _ in 0..q:
    let (s_raw, t_raw): (V, V) = read()
    let (src, dst) = (comp[s_raw], comp[t_raw])
    var keys = [src, dst]
    var extra: List<(u: C, v: C)> = []
    for _ in 0..kv:
        let (eu, ev): (V, V) = read()
        let (cu, cv) = (comp[eu], comp[ev])
        extra.push((u: cu, v: cv))
        keys.push(cu)
        keys.push(cv)

    let vt = tree.virtual(keys)
    for u in vt.at:
        fwd[u] = []
        rev[u] = []
        vis_f[u] = false
        vis_r[u] = false
    for x in vt.K:
        if let p = vt.parent[x]:
            fwd[vt.at[p]].push(vt.at[x])
            rev[vt.at[x]].push(vt.at[p])
    for e in extra:
        fwd[e.u].push(e.v)
        rev[e.v].push(e.u)

    // Forward BFS from src.
    vis_f[src] = true
    var qf = [src]
    var head = 0
    while head < qf.len:
        let u = qf[head]
        head += 1
        for v in fwd[u]:
            if not vis_f[v]:
                vis_f[v] = true
                qf.push(v)

    // Backward BFS from dst.
    vis_r[dst] = true
    var qr = [dst]
    head = 0
    while head < qr.len:
        let u = qr[head]
        head += 1
        for v in rev[u]:
            if not vis_r[v]:
                vis_r[v] = true
                qr.push(v)

    var ans = 0
    for x in vt.K:
        let u = vt.at[x]
        if vis_f[u] and vis_r[u]: ans += wt[u]
        if let p = vt.parent[x], vis_f[vt.at[p]] and vis_r[u]: ans += vt.w[x] - wt[u]
    print(ans)
