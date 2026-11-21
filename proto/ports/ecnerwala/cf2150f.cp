@!multitest
let n: 3..=200 = read()
let m: 2..=19_900 = read()
domain V = 1..=n
var adj: [V, V]Bool = false
repeat m:
    let (u, v): (V, V) = read()
    adj[u, v] = true
    adj[v, u] = true

var tree: [V]List<V> = []
var vis: [V]Bool = false
var diam = 0
fn build_tree(cur: V) -> Int:
    vis[cur] = true
    var d = 0
    for nxt in V:
        if not vis[nxt] and adj[cur, nxt]:
            tree[cur].push(nxt)
            tree[nxt].push(cur)
            let nd = build_tree(nxt) + 1
            diam max= d + nd
            d max= nd
    d

let _ = build_tree(V.first)

print(2)
print(3)
var ops1: List<(V, V, V)> = []
for i in V:
    for j in tree[i]:
        for k in tree[i]:
            if j == k or adj[j, k]: continue
            adj[j, k] = true
            adj[k, j] = true
            ops1.push((j, i, k))
print(ops1.len)
for (a, b, c) in ops1: print(a, b, c)

let path_len = ceil_div(diam, 2)
print(path_len + 1)
var num_ops = 0
for i in V:
    for j in V:
        if j > i and not adj[i, j]: num_ops += 1
print(num_ops)

fn furthest_from(cur: V, prv: ?V) -> (Int, V):
    var best = (-1, cur)
    for nxt in tree[cur]:
        if nxt == prv: continue
        best max= furthest_from(nxt, cur)
    (best.0 + 1, best.1)

fn tree_path(st: V, en: V) -> List<V>:
    var path: List<V> = []
    fn dfs(cur: V, prv: ?V) -> Bool:
        path.push(cur)
        if cur == en: return true
        for nxt in tree[cur]:
            if nxt == prv: continue
            if dfs(nxt, cur): return true
        let _ = path.pop()
        false
    let _ = dfs(st, none)
    path

for i in V:
    let furthest = furthest_from(i, none).1
    for j in V:
        if j <= i or adj[i, j]: continue
        var resp: List<V> = []
        let pij = tree_path(i, j)
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
            var pif = tree_path(i, furthest)
            var pjf = tree_path(j, furthest)
            var extra = (pif.len + pjf.len - 2 - dij) / 2
            let need = path_len - dij
            while extra > need:
                let _ = pif.pop()
                let _ = pjf.pop()
                extra -= 1
            var extras: List<V> = []
            repeat extra + 1:
                extras.push(pif.pop()!)
                let _ = pjf.pop()
            extras.reverse()
            let f = if pjf.is_empty: 1 else: 0
            for x in pif: resp.push(x)
            for z in 0..extras.len:
                if z % 2 == f: resp.push(extras[z])
            var par = (extras.len - 1) % 2
            for x in extras.rev:
                if par != f: resp.push(x)
                par = 1 - par
            for x in pjf.rev: resp.push(x)
        print(resp)
