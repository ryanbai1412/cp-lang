// Source: AtCoder ABC479 E (Cut and Add), https://atcoder.jp/contests/abc479/tasks/abc479_e
// Problem: a tree with an odd number N <= 2e5 of vertices, edges numbered 1..N-1. For each edge
//   i, removing it splits the tree; add i to x_v for every v in the larger part. Print x_1..x_N.
// Solution: root at 1. The edge above child c adds i to sub(c) when size(c) > N - size(c), else to
//   everything outside sub(c): add i to a base and -i at c. Pushing the offsets down in preorder
//   gives every x in O(N); the @brute redoes each cut by DFS in O(N^2).

let n: 3..=2e5 = read()
domain V = 1..=n
domain E = 1..n
let edges: [E](u: V, v: V) = read()

let g = Graph.undirected(edges)
let t = Rooted(g, V.first!)

var base = 0
var diff: [V]Int = 0
for e in E:
    let u = edges[e].u
    let v = edges[e].v
    let id = Int(e)
    let c = if t.parent(u) == v: u else: v
    let s = t.size(c)
    if s > n - s:
        diff[c] += id
    else:
        base += id
        diff[c] -= id

var ans: [V]Int = base
for u in t.at:
    if let p = t.parent(u):
        diff[u] += diff[p]
    ans[u] += diff[u]

print(ans)

@alt(program)
fn slow():
    var adj: [V]List<(V, Int)> = []
    for e in E:
        let u = edges[e].u
        let v = edges[e].v
        let id = Int(e)
        adj[u].push((v, id))
        adj[v].push((u, id))
    var out: [V]Int = 0
    for ban in 1..n:
        var vis: [V]Bool = false
        var comp: List<V> = []
        fn dfs(u: V):
            vis[u] = true
            comp.push(u)
            for nb in adj[u]:
                if nb.1 != ban and not vis[nb.0]:
                    dfs(nb.0)
        dfs(V.first!)
        if comp.len > n - comp.len:
            for u in comp: out[u] += ban
        else:
            for u in V:
                if not vis[u]: out[u] += ban
    print(out)
