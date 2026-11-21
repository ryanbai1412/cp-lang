@!multitest
let n: Int = read()
var g: [0..n, 0..n]Int = 0
fn coord(a: Int) -> Int = if (a & 1) == 1: n / 2 - 1 - a / 2 else: n / 2 + a / 2
var p = 0
for i in 0..n:
    for j in 0..i:
        g[coord(i), coord(j)] = p
        p += 1
    for j in 0..=i:
        g[coord(j), coord(i)] = p
        p += 1
for i in 0..n:
    print([g[i, j] for j in 0..n])
