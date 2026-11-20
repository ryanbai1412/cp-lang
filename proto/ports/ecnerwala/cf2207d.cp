// Codeforces 2207 D: root the tree at V and compute the minimum distance to a
// leaf in each subtree, returning 0 as soon as two leaves lie within K + 1 edges.
@!multitest
let (n, k, v_in): (Int, Int, Int) = read()
let v = v_in - 1
var adj: List<List<Int>> = List.filled(n, [])
for _ in 0..n - 1:
    let (u, w): (Int, Int) = read()
    adj[u - 1].push(w - 1)
    adj[w - 1].push(u - 1)

fn dfs(cur: Int, prv: ?Int) -> Int:
    if adj[cur].len == 1: return 0
    var d = k + 2
    for nxt in adj[cur]:
        if some(nxt) == prv: continue
        let nd = dfs(nxt, cur) + 1
        if d + nd <= k + 1: return 0
        d min= nd
    d

print(if dfs(v, none) == 0: "YES" else: "NO")
