@!multitest
let k: 2..=2e5 = read()
var hub = (1, 1)
var found = false
for tree in 1..=k:
    let n: 1..=2e5 = read()
    var deg = [0 for _ in 0..=n]
    var edges: List<(Int, Int)> = []
    for _ in 0..n - 1:
        let (u, v): (1..=n, 1..=n) = read()
        deg[u] += 1
        deg[v] += 1
        edges.push((u, v))
    if not found and n >= 2:
        let leaf = (1..=n).find(x => deg[x] == 1)!
        let (p, q) = edges.find(e => e.0 == leaf or e.1 == leaf)!
        hub = (tree, if p == leaf: q else: p)
        found = true
for tree in 1..=k:
    if tree != hub.0:
        print(tree, 1, hub.0, hub.1)
