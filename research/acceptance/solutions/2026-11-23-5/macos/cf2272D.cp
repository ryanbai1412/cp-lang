// D. Connecting the Conflicts
// https://codeforces.com/contest/2272/problem/D
// judge: generic

@!multitest
let k: 2..=2e5 = read()
var hub = (1, 1)
var have_hub = false
for t in 1..=k:
    let n: 1..=2e5 = read()
    var deg = [0 for _ in 0..=n]
    var edges: List<(Int, Int)> = []
    for _ in 0..n - 1:
        let (u, v): (1..=n, 1..=n) = read()
        deg[u] += 1
        deg[v] += 1
        edges.push((u, v))
    if not have_hub and n >= 2:
        for (u, v) in edges:
            if deg[u] == 1:
                hub = (t, v)
                break
            if deg[v] == 1:
                hub = (t, u)
                break
        have_hub = true
for t in 1..=k:
    if t != hub.0: print(t, 1, hub.0, hub.1)
