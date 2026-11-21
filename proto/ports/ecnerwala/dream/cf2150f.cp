// Codeforces 2150 F: Cycle Closing (dream).
// Build a spanning tree T of diameter d.
// Op 1 (k = 3): connect every pair of tree neighbors to form T^2.
// Op 2 (k = ceil(d / 2) + 1): connect every remaining non-edge (i, j) by a
// simple path of length path_len in T^2, skipping along T.path(i, j) when long
// enough or zigzagging out and back toward the further diameter endpoint.
with io(base: 1)

--- with multitest() ---

domain V = read(3..=200)
let m: 2..=19_900 = read()
var adj: [V, V]Bool = false
var tree: [V]List<V> = []
var dsu = Dsu<V>()
repeat m:
    let (u, v): (V, V) = read()
    adj[u, v] = adj[v, u] = true
    if dsu.unite(u, v):
        tree[u].push(v)
        tree[v].push(u)

let T = Rooted(tree, V.first)
let d = T.diameter()

print(2)
print(3)
var ops1: List<(V, V, V)> = []
for i in V:
    for j in tree[i], k in tree[i] where j != k and not adj[j, k]:
        adj[j, k] = adj[k, j] = true
        ops1.push((j, i, k))
print(ops1.len)
for (a, b, c) in ops1: print(a, b, c)

let path_len = ceil_div(d.len, 2)
print(path_len + 1)
print(count(for i in V, j in V.after(i) where not adj[i, j]))

for i in V:
    let far = if T.dist(i, d.u) >= T.dist(i, d.v): d.u else: d.v
    for j in V.after(i) where not adj[i, j]:
        let pij = T.path(i, j)
        let dij = pij.len - 1
        var resp: List<V> = []
        if dij >= path_len:
            var skip = dij - path_len
            var cur = 0
            repeat path_len:
                resp.push(pij[cur])
                if skip > 0: cur += 1; skip -= 1
                cur += 1
            resp.push(pij[cur])
        else:
            var pif = T.path(i, far)
            var pjf = T.path(j, far)
            let extra = (pif.len + pjf.len - 2 - dij) / 2
            let drop = extra - (path_len - dij)
            pif.truncate(pif.len - drop)
            pjf.truncate(pjf.len - drop)
            let keep = path_len - dij + 1
            let extras = pif[pif.len - keep..]
            pif.truncate(pif.len - keep)
            pjf.truncate(pjf.len - keep)
            let f = if pjf.is_empty: 1 else: 0
            resp.extend(pif)
            for (z, x) in extras.items where z % 2 == f: resp.push(x)
            for (z, x) in extras.items.rev where z % 2 != f: resp.push(x)
            resp.extend(pjf.rev)
        print(resp)
