let (n, m): (1..=1000, 1..=1000) = read()
var g: List<List<Int>> = [[0 for _ in 0..m] for _ in 0..n]
g[n - 1][m - 1] = 7
print(g[n - 1][m - 1])
