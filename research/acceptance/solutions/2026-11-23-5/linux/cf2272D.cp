@!multitest
let k: 2..=2e5 = read()
var hub = (1, 1)
var found = false
for t in 1..=k:
    let n: 1..=2e5 = read()
    var deg = [n + 1].filled(0)
    var other = [n + 1].filled(0)
    for _ in 0..n - 1:
        let (u, v): (1..=n, 1..=n) = read()
        deg[u] += 1
        deg[v] += 1
        other[u] = v
        other[v] = u
    if not found and n >= 2:
        let leaf = (1..=n).first(v => deg[v] == 1)!
        hub = (t, other[leaf])
        found = true
for t in 1..=k:
    if t != hub.0: print(t, 1, hub.0, hub.1)
