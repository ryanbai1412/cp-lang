let (n, m): (1..=1000, 1..=1000) = read()
var g: [n, m]Int = 0
g[n - 1, m - 1] = 7
print(g[n - 1, m - 1])
