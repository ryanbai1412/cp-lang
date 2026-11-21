// Codeforces 2183 D1: bucket tree vertices by depth; answer is max over depths of
// layer size, plus 1 when every vertex in a non-root layer shares one parent.
@!multitest
let n: Int = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let t = Rooted(Graph.undirected(edges), root: V(1))

var by_depth: [0..n]List<V> = []
for v in V: by_depth[t.depth(v)].push(v)

var ans = 0
for d in 0..n:
    if by_depth[d].is_empty: break
    let p0 = t.parent(by_depth[d][0])
    let same_parent = d > 0 and all(t.parent(v) == p0 for v in by_depth[d])
    ans max= by_depth[d].len + Int(same_parent)
print(ans)
