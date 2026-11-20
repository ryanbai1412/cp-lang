// Codeforces 2222 G: root at a tree centroid, aggregate non-dominant subtree
// sizes by frequency at each vertex, and merge centroid-excluded vs non-excluded
// component sizes via prefix sums.
@!multitest
let n: Int = read()
var adj: List<List<Int>> = List.filled(n, [])
for _ in 0..n - 1:
    let (u, v): (Int, Int) = read()
    adj[u - 1].push(v - 1)
    adj[v - 1].push(u - 1)

var sz: [0..n]Int = 1

fn dfs_sz(cur: Int, prv: ?Int):
    sz[cur] = 1
    for nxt in adj[cur]:
        if some(nxt) == prv: continue
        dfs_sz(nxt, cur)
        sz[cur] += sz[nxt]

dfs_sz(0, none)
var rt = 0
while true:
    var n_rt: ?Int = none
    for nxt in adj[rt]:
        if sz[nxt] > sz[rt]: continue
        if sz[nxt] * 2 > n: n_rt = nxt
    if let next_root = n_rt: rt = next_root
    else: break

dfs_sz(rt, none)

var ans: [0..=n]Int = 0
ans[n] = n

var excl_sz: [0..=n]Int = 0
var easy_sz: [0..=n]Int = 0

fn collect_excl(cur: Int, prv: Int, val: Int):
    excl_sz[max(val, sz[cur])] += 1
    for nxt in adj[cur]:
        if nxt == prv: continue
        collect_excl(nxt, cur, max(val, sz[cur] - sz[nxt]))

fn collect_easy(cur: Int, prv: Int, val: Int):
    easy_sz[max(val, sz[cur])] += 1
    for nxt in adj[cur]:
        if nxt == prv: continue
        collect_easy(nxt, cur, max(val, sz[cur] - sz[nxt]))

for v in 0..n:
    if n == 1: continue
    var excl = adj[v][0]
    for nxt in adj[v]:
        if sz[nxt] > sz[excl]: excl = nxt
    var mp: Map<Int, Int> = Map()
    for nxt in adj[v]:
        if nxt == excl: continue
        mp[sz[nxt]] = (mp.get(sz[nxt]) ?? 0) + 1
    let freqs = [(k, c) for (k, c) in mp]
    for i in 0..freqs.len:
        let (s1, c1) = freqs[i]
        ans[n - s1] += s1 * c1
        if c1 >= 2:
            ans[n - 2 * s1] += (c1 * (c1 - 1) / 2) * s1 * s1
        for j in i + 1..freqs.len:
            let (s2, c2) = freqs[j]
            ans[n - s1 - s2] += (s1 * c1) * (s2 * c2)
    if v == rt:
        ans[n - sz[excl]] += sz[excl]
        collect_excl(excl, rt, 0)
        for ch in adj[v]:
            if ch == excl: continue
            collect_easy(ch, rt, n - sz[excl] - sz[ch])
        var p_excl = 0
        var p_easy = 0
        for i in 0..=n:
            ans[i] += p_excl * easy_sz[i] + p_easy * excl_sz[i] + easy_sz[i] * excl_sz[i]
            p_excl += excl_sz[i]
            p_easy += easy_sz[i]

print(ans[1..=n])
