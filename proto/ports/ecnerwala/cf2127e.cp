@!multitest
let (n, k): (Int, Int) = read()
domain V = 0..n
let w: [V]Int = [read::<Int>() for _ in V]
var col: [V]Int = [read::<Int>() - 1 for _ in V]
var adj: [V]List<V> = []
for _ in 0..(n - 1):
    let (u, v): (Int, Int) = read()
    let (a, b) = (V(u - 1), V(v - 1))
    adj[a].push(b)
    adj[b].push(a)
let tree = Rooted(adj, V(0))
var by_col: [0..k]List<(Int, V)> = []
for v in V:
    if col[v] != -1:
        by_col[col[v]].push((tree.tin[v].ord, v))
var is_bad: [V]Bool = false
for c in 0..k:
    by_col[c].sort()
    let m = by_col[c].len
    for i in 0..(m - 1):
        let l = tree.lca(by_col[c][i].1, by_col[c][i + 1].1)
        if col[l] == -1: col[l] = c
        if col[l] != c: is_bad[l] = true
var ans = 0
for v in V:
    if is_bad[v]: ans += w[v]
for v in tree.at:
    if col[v] == -1:
        col[v] = if let p = tree.par[v]: col[p] else: 0
print(ans)
print([col[v] + 1 for v in V])
