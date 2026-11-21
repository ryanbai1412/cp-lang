// Codeforces 2245 D2: every vertex gets a non-zero value; an edge of kind 0
// or 1 constrains the sign of the sum of its endpoints.  Peel vertices that
// only have edges of one kind left: the z-th peeled vertex gets magnitude
// n - z, large enough to dominate everything peeled later.
@!multitest
let (n, m): (1..=2e5, 0..=2e5) = read()
let edges: [0..m](kind: 1..=2, u: 1..=n, v: 1..=n) = read()
var adj: [0..n]List<Int> = []
var cnt: [0..n, 0..2]Int = 0
for (e, ed) in edges.items:
    for x in [ed.u - 1, ed.v - 1]:
        adj[x].push(e)
        cnt[x, ed.kind - 1] += 1

var order = [v for v in 0..n if cnt[v, 0] == 0 or cnt[v, 1] == 0]
var queued = [0..n] v => cnt[v, 0] == 0 or cnt[v, 1] == 0
var ans: [0..n]Int = 0
var z = 0
while z < order.len:
    let v = order[z]
    ans[v] = if cnt[v, 0] == 0: z - n else: n - z
    for e in adj[v]:
        let ed = edges[e]
        let u = ed.u - 1 + ed.v - 1 - v
        if queued[u]: continue
        cnt[u, ed.kind - 1] -= 1
        if cnt[u, 0] == 0 or cnt[u, 1] == 0:
            order.push(u)
            queued[u] = true
    z += 1

if order.len < n:
    print("NO")
    return
print("YES")
print(ans)
