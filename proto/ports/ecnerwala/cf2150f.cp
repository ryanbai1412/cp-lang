@!multitest
let n: 3..=200 = read()
let m: 2..=19_900 = read()
domain V = 1..=n
var adj: [V, V]Bool = false
var tree: [V]List<V> = []
var dsu = Dsu<V>()
repeat m:
    let (u, v): (V, V) = read()
    adj[u, v] = true
    adj[v, u] = true
    if dsu.unite(u, v):
        tree[u].push(v)
        tree[v].push(u)

let T = Rooted(tree, V.first)
let d = T.diameter()

print(2)
print(3)
var ops1: List<(V, V, V)> = []
for i in V:
    for j in tree[i]:
        for k in tree[i] if j != k and not adj[j, k]:
            adj[j, k] = true
            adj[k, j] = true
            ops1.push((j, i, k))
print(ops1.len)
for (a, b, c) in ops1: print(a, b, c)

let path_len = ceil_div(d.len, 2)
print(path_len + 1)
var num_ops = 0
for i in V:
    for j in V if j > i and not adj[i, j]: num_ops += 1
print(num_ops)

for i in V:
    let far = if T.dist(i, d.u) >= T.dist(i, d.v): d.u else: d.v
    for j in V if j > i and not adj[i, j]:
        var resp: List<V> = []
        let pij = T.path(i, j)
        let dij = pij.len - 1
        if dij >= path_len:
            var skip = dij - path_len
            var cur = 0
            repeat path_len:
                resp.push(pij[cur])
                if skip > 0:
                    cur += 1
                    skip -= 1
                cur += 1
            resp.push(pij[cur])
        else:
            var pif = T.path(i, far)
            var pjf = T.path(j, far)
            let extra = (pif.len + pjf.len - 2 - dij) / 2
            let need = path_len - dij
            let drop = extra - need
            pif.truncate(pif.len - drop)
            pjf.truncate(pjf.len - drop)
            let keep = need + 1
            let extras = [pif[k] for k in pif.len - keep..pif.len]
            pif.truncate(pif.len - keep)
            pjf.truncate(pjf.len - keep)
            let f = if pjf.is_empty: 1 else: 0
            resp.extend(pif)
            for (z, x) in extras.items if z % 2 == f: resp.push(x)
            for (z, x) in extras.items.rev if z % 2 != f: resp.push(x)
            resp.extend(pjf.rev)
        print(resp)
