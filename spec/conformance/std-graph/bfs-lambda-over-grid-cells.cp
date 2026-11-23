// expect: ok, output "3"
// input: "2 3\n..#\n...\n"
// A lambda graph over grid cells: the vertex type `(R, C)` comes from `from:` (rebuild round 6 H-BFS-LAMBDA-VERTEX).
let (r, c) = read<(Int, Int)>()
domain R = 0..r
domain C = 0..c
let grid = read<[R, C]Char>()
let s = (R(0), C(0))
let d = bfs(p => p.adj4.filter(q => grid[q] != ('#')), from: [s])
print(d[(R(1), C(2))])
