// SAC '22 Code Challenge 2 P4: walk from the top-left to the bottom-right cell of a grid,
// stepping to any of the four neighbours, through as few 'C' cells as possible.
// DRAFT 1: the classic grid DP, coming only from above or from the left.
let n: 1..=1000 = read()
let m: 1..=1000 = read()
domain Row = 1..=n
domain Col = 1..=m

fn random_grid() -> [Row, Col]Char:
    [Row, Col] (r, c) => if rand(0, 2) == 0: '.' else: 'C'

let grid: [Row, Col]Char = read(gen: random_grid)

fn cost(r: Row, c: Col) -> Int: Int(grid[r, c] == 'C')

var best: [Row, Col]Int = 0
for r in Row:
    for c in Col:
        var prev: ?Int = none
        if let p = r - 1: prev = best[p, c]
        if let p = c - 1: prev = min(prev ?? best[r, p], best[r, p])
        best[r, c] = cost(r, c) + (prev ?? 0)
print(best[Row.last, Col.last])
