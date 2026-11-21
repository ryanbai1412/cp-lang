type Z = Mod<998_244_353>

@! multitest
let (n, m, lim): (Int, Int, Int) = read()
domain V = 1..=n
domain E = 0..m
let raw_a: [V]Int = read()
let edges: [E](u: V, v: V) = read()
let g = Graph.undirected(edges)

var depth: [V]?Int = none
var low: [V]Int = 0
var must: [V]?Int = [V] u => if raw_a[u] < 0: none else: raw_a[u]
var ans: Z = 1

fn force(u: V, val: Int):
    if let prev = must[u]:
        if prev != val: ans = 0
    else:
        must[u] = val

fn dfs(u: V, p: ?V, d: Int):
    depth[u] = d
    low[u] = d
    for a in g.adj(u):
        let v = a.to
        if p == v: continue
        if let nd = depth[v]:
            if nd < d:
                low[u] min= nd
                if (d - nd) % 2 == 0: force(u, 0)
        else:
            dfs(v, u, d + 1)
            if low[v] <= d:
                low[u] min= low[v]
                if let mv = must[v]: force(u, mv)
    if low[u] == d and must[u] == none:
        ans *= Z(lim)

dfs(V(1), none, 0)
print(ans)
