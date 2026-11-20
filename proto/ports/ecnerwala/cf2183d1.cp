// Codeforces 2183 D1: bucket tree vertices by depth; answer is max over depths of
// layer size, plus 1 when every vertex in a non-root layer shares one parent.
@!multitest
let n: Int = read()
var adj: List<List<Int>> = List.filled(n, [])
for _ in 0..n - 1:
    let (u, v): (Int, Int) = read()
    adj[u - 1].push(v - 1)
    adj[v - 1].push(u - 1)

var by_depth: List<List<(Int, ?Int)>> = List.filled(n, [])

fn dfs(cur: Int, prv: ?Int, depth: Int):
    by_depth[depth].push((cur, prv))
    for nxt in adj[cur]:
        if some(nxt) == prv: continue
        dfs(nxt, cur, depth + 1)

dfs(0, none, 0)
var ans = 0
for d in 0..n:
    if by_depth[d].is_empty: break
    let sz = by_depth[d].len
    let p0 = by_depth[d][0].1
    let same_parent = d > 0 and all(p == p0 for (_, p) in by_depth[d])
    ans max= sz + Int(same_parent)
print(ans)
