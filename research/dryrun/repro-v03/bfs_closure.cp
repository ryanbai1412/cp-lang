// std.grid's own example: a grid is a graph by writing its moves. `cpl check` accepts; `cpl run` rejects.
domain R = 0..2
domain C = 0..2
let g = [R, C] (r, c) => if r == R(1) and c == C(0): '#' else: '.'
let p = bfs(c => c.adj4.filter(q => g[q] != '#'), from: [(R(0), C(0))])
print(p[(R(1), C(1))]!)
