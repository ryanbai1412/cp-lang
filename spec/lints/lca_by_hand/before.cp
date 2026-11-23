let n: 2..=1000 = read()
var parent: List<Int> = [0]
var depth: List<Int> = [0]
for v in 1..n:
    let p: 0..v = read()
    parent.push(p)
    depth.push(depth[p] + 1)
let (u0, v0): (0..n, 0..n) = read()
var u = u0
var v = v0
while depth[u] > depth[v]:
    u = parent[u]
while depth[v] > depth[u]:
    v = parent[v]
while u != v:
    u = parent[u]
    v = parent[v]
print(u)
