@!multitest
let k: 2..=2e5 = read()
var hub: ?(Int, Int) = None
for t in 1..=k:
    let n: 1..=2e5 = read()
    let edges: [0..n - 1](1..=n, 1..=n) = read()
    if n >= 2 and hub == None:
        var deg = [0 for _ in 0..=n]
        for (u, v) in edges:
            deg[u] += 1
            deg[v] += 1
        for (u, v) in edges:
            if deg[u] == 1:
                hub = Some((t, v))
                break
            if deg[v] == 1:
                hub = Some((t, u))
                break
let (ht, hv) = hub ?? (1, 1)
for t in 1..=k:
    if t != ht:
        print(t, 1, ht, hv)
