let n: 2..=200000 = read()
let par: [0..n - 1]1..=n = read()
var depth: List<Int> = [0, 0]
for v in 2..=n:
    depth.push(depth[par[v - 2]] + 1)
print(depth.max()!)
