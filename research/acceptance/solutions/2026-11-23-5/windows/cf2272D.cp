// D. Connecting the Conflicts
// https://codeforces.com/contest/2272/problem/D
// the neighbour p of a leaf is avoided by some maximum independent set, so hanging every other tree
// off p keeps the sum of the maximum independent sets; with only single vertices, a star loses one,
// which no tree on k vertices avoids
@!multitest
let k: 2..=2e5 = read()
var hub: ?(Int, Int) = None
for i in 1..=k:
    let n: 1..=2e5 = read()
    let edges: [0..n - 1](1..=n, 1..=n) = read()
    var deg: [0..=n]Int = 0
    for (u, v) in edges:
        deg[u] += 1
        deg[v] += 1
    if hub == None:
        for (u, v) in edges:
            if deg[u] == 1:
                hub = (i, v)
            elif deg[v] == 1:
                hub = (i, u)
let (h, p) = hub ?? (1, 1)
for j in 1..=k:
    if j != h: print(h, p, j, 1)

@check
fn check():
    for _ in 1..k:
        let (t1, v1, t2, v2): (Int, Int, Int, Int) = read_output()
