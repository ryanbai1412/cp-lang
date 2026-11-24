let n: 2..=200000 = read()
let par: [0..n - 1]1..=n = read()
var depth: List<Int> = [0, 0]
for v in 2..=n:
    depth.push(depth[par[v - 2]] + 1)
let (a, b): (1..=n, 1..=n) = read()
print(if depth[a] >= depth[b]: a else: b)
