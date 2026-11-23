// CCO 2016 P4, O Canada: a move flips all four cells of a 2 x 2 square of a
// red/white n x n grid. Count the pairs of grids that moves can turn into
// each other.
// original: corpus/submissions/3605322.cpp
let n: 2..=10 = read()
let g: 1..=100 = read()
domain Row = 0..n
domain Col = 0..n
type Grid = [Row, Col]Char

fn random_grid() -> Grid:
    [Row, Col] (r, c) => if rand(0, 1) == 0: 'R' else: 'W'

// A move keeps the number of red cells in every row and every column even
// or odd, and those parities are all it keeps: grids match when they agree.
var seen: Counter<(List<Int>, List<Int>), Int> = Counter(0)
var pairs = 0
for _ in 0..g:
    let grid: Grid = read(gen: random_grid)
    let rows = [count(grid[r, c] == 'R' for c in Col) % 2 for r in Row]
    let cols = [count(grid[r, c] == 'R' for r in Row) % 2 for c in Col]
    let key = (rows, cols)
    pairs += seen[key]
    seen[key] += 1
print(pairs)
