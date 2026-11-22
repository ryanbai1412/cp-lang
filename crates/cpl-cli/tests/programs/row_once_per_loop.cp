// A row reached by indices a loop never changes is checked once per loop,
// and only when the loop reaches it.
let n: 1..=50 = read()
var g = [[0 for _ in 0..n] for _ in 0..n]
for i in 0..n:
    for j in 0..n:
        g[i][j] = i * n + j
var total = 0
let far = n + 5
for j in 0..0:
    total += g[far][j]
for i in 0..n:
    for j in 0..n:
        total += g[i][j]
        if j == 0: g.push([i])
print(total, g.len, g[n][0])
