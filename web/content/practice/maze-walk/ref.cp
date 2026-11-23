let (n, m): (1..=1000, 1..=1000) = read()
domain R = 0..n
domain Col = 0..m
let grid: [R, Col]Char = read()
let p = bfs(c => [q for q in c.adj4 if grid[q] != '#'], [(R(0), Col(0))])
print(p[(R(n - 1), Col(m - 1))] ?? -1)
