let n: 2..=200000 = read()
let par: [0..n - 1]1..=n = read()
var size: List<Int> = [0 for _ in 0..=n]
for v in 2..=n:
    size[par[v - 2]] = size[par[v - 2]] + 1
print(size[1])
