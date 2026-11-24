// Source: CCO 2016 Day 2 (O Canada), https://dmoj.ca/problem/cco16p4
// limits: 600 ms, 1024 MB
// Problem: 2 <= G <= 1e4 grids of n x n red/white cells (2 <= n <= 10). A change flips the
//   four cells of a 2 x 2 square. Count the pairs of grids that changes can turn into each other.
// Solution: a change keeps the parity of red cells in every row and every column, and these
//   2n - 1 independent parities are all it keeps ((n-1)^2 squares span the rest). So key each
//   grid by its row and column parities and count equal pairs with a Counter: O(G n^2).
// original: corpus/submissions/3605322.cpp
// Bounds: n: 2..=10; g: 2..=10_000
let n: 2..=10 = read()
let g: 2..=10_000 = read()
domain Row = 0..n
domain Col = 0..n

fn red(ch: Char) -> Int = Int(ch == 'R')

var seen: Counter<(List<Int>, List<Int>), Int> = Counter()
var pairs = 0
for _ in 0..g:
    let grid: [Row, Col]Char = read()
    let key = ([sum(red(grid[r, c]) for c in Col) % 2 for r in Row],
               [sum(red(grid[r, c]) for r in Row) % 2 for c in Col])
    pairs += seen[key]
    seen[key] += 1
print(pairs)
