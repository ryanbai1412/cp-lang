// AtCoder DP H Grid 1: count the paths from the top-left to the bottom-right
// corner that move right or down through open squares.
type Z = Mod<1_000_000_007>
let (h, w): (2..=1000, 2..=1000) = read()
domain Row = 1..=h
domain Col = 1..=w
let grid: [Row, Col]Char = read()

var ways: [Row, Col]Z = 0
ways[Row.first!, Col.first!] = 1
for r in Row:
    for c in Col if grid[r, c] == '.':
        if let up = Row.at(r - 1): ways[r, c] += ways[up, c]
        if let left = Col.at(c - 1): ways[r, c] += ways[r, left]
print(ways[Row.last!, Col.last!])
