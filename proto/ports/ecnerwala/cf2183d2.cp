// Codeforces 2183 D2: compute the minimum number of operations by depth, then assign
// vertices bottom-up into slots so no parent shares a slot with its child.
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

var resp: List<List<Int>> = List.filled(ans, [])
var last_depth: [0..ans]Int = n + 1
var forbidden_loc: [0..n]?Int = none

for d in (0..n).rev:
    if by_depth[d].is_empty: continue
    var forbiddens: List<(Int, ?Int, Int)> = []
    for (cur, prv) in by_depth[d]:
        if let fl = forbidden_loc[cur]:
            forbiddens.push((cur, prv, fl))
    if forbiddens.len == 1:
        let (cur, prv, _) = forbiddens[0]
        var loc = 0
        while last_depth[loc] == d + 1: loc += 1
        resp[loc].push(cur + 1)
        last_depth[loc] = d
        if let p = prv: forbidden_loc[p] = loc
    else:
        let k = forbiddens.len
        for z in 0..k:
            let (cur, prv, _) = forbiddens[z]
            let loc = forbiddens[(z + 1) % k].2
            resp[loc].push(cur + 1)
            last_depth[loc] = d
            if let p = prv: forbidden_loc[p] = loc
    var loc = 0
    for (cur, prv) in by_depth[d]:
        if forbidden_loc[cur] == none:
            while last_depth[loc] == d: loc += 1
            resp[loc].push(cur + 1)
            last_depth[loc] = d
            if let p = prv: forbidden_loc[p] = loc

print(ans)
for group in resp:
    print(group.len, group)
