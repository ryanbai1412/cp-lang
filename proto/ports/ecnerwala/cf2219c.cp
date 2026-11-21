@!multitest
let n: Int = read()
let s: Str = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let g = Graph.undirected(edges)

var rt = V(1)
for v in V if s[v.to_idx] == '1':
    rt = v
    break
let t = Rooted(g, root: rt)

let inf = 1e99
var dp0: [V]F64 = 0.0
var dp1: [V]F64 = 0.0
for u in t.postorder:
    var base = 0.0
    var ch: List<F64> = []
    for c in t.children(u):
        ch.push(dp1[c] - dp0[c])
        base += dp0[c]
    if s[u.to_idx] == '1':
        dp0[u] = base
        dp1[u] = base
        continue
    ch.sort()
    let deg = F64(g[u].len)
    var b0 = base + deg
    var b1 = inf
    for (i, x) in ch.items:
        base += x
        b1 min= base + deg / F64(i + 1)
        b0 min= base + deg / F64(i + 2)
    dp0[u] = b0
    dp1[u] = b1

print(dp0[rt])
