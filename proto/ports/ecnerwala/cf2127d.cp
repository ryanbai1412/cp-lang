@!multitest
type Z = Mod<1_000_000_007>
let (n, m): (Int, Int) = read()
var edges: List<(Int, Int)> = []
for _ in 0..m:
    let (u, v): (Int, Int) = read()
    edges.push((u - 1, v - 1))
if m > n - 1:
    print(0)
    return
var deg: [0..n]Int = 0
for (u, v) in edges:
    deg[u] += 1
    deg[v] += 1
var deg2: [0..n]Int = 0
var num_e = 0
for (u, v) in edges:
    if deg[u] == 1 or deg[v] == 1: continue
    deg2[u] += 1
    deg2[v] += 1
    num_e += 1
if any(deg2[i] >= 3 for i in 0..n):
    print(0)
    return
if num_e >= 1:
    var ans: Z = 4
    for i in 0..n:
        if deg[i] == 1: continue
        for z in 1..=(deg[i] - deg2[i]):
            ans *= z
    print(ans)
else:
    var ans: Z = 2
    for i in 1..n:
        ans *= i
    print(ans)
