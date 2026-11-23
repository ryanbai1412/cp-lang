let (h, w): (1..=1000, 1..=1000) = read()
domain R = 0..h
domain C = 0..w
let grid: [R, C]Char = read()
let start = grid.items.find(e => e.1 == 'S')!.0
let goal = grid.items.find(e => e.1 == 'E')!.0
let p = bfs(c => [q for q in c.adj4 if grid[q] != '#'], [start])
print(p[goal] ?? -1)
