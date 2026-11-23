@!multitest
let k: 2..=2e5 = read()
var hub = -1
var hubv = 1
for t in 0..k:
    let n: 1..=2e5 = read()
    let es: [0..n - 1](1..=n, 1..=n) = read()
    if hub < 0 and n >= 2:
        var deg = [0 for _ in 0..=n]
        for (u, v) in es:
            deg[u] += 1
            deg[v] += 1
        for (u, v) in es:
            if deg[u] == 1:
                hubv = v
                break
            if deg[v] == 1:
                hubv = u
                break
        hub = t
if hub < 0: hub = 0
for t in 0..k:
    if t != hub: print(hub + 1, hubv, t + 1, 1)
