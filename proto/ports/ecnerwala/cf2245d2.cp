// Codeforces 2245 D2: every vertex gets a non-zero value; an edge of kind 0
// or 1 constrains the sign of the sum of its endpoints.  Peel vertices that
// only have edges of one kind left: the z-th peeled vertex gets magnitude
// n - z, large enough to dominate everything peeled later.
@!multitest
let (n, m): (1..=2e5, 0..=2e5) = read()
let edges: [0..m](kind: 1..=2, u: 1..=n, v: 1..=n) = read()
var adj: [0..n]List<Int> = []
var cnt = [[0, 0] for _ in 0..n]
for e in 0..m:
    for x in [edges[e].u - 1, edges[e].v - 1]:
        adj[x].push(e)
        cnt[x][edges[e].kind - 1] += 1
var order = [v for v in 0..n if cnt[v][0] == 0 or cnt[v][1] == 0]
var queued = [cnt[v][0] == 0 or cnt[v][1] == 0 for v in 0..n]
var ans: [0..n]Int = 0
var z = 0
while z < order.len:
    let v = order[z]
    ans[v] = if cnt[v][0] == 0: z - n else: n - z
    for e in adj[v]:
        let u = edges[e].u - 1 + edges[e].v - 1 - v
        if queued[u]: continue
        cnt[u][edges[e].kind - 1] -= 1
        if cnt[u][0] == 0 or cnt[u][1] == 0:
            order.push(u)
            queued[u] = true
    z += 1
if order.len < n:
    print("NO")
    return
print("YES")
print(ans)
