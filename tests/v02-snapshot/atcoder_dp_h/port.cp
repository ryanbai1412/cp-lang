// Source: AtCoder Educational DP Contest H (Grid 1), https://atcoder.jp/contests/dp/tasks/dp_h
// Problem: an h x w grid (h, w <= 1000) of '.' and '#' walls, both corners open: count the
//   paths from the top-left to the bottom-right square that move right or down through open
//   squares, mod 1e9+7.
// Solution: ways[r, c] = ways[r - 1, c] + ways[r, c - 1] on an open square (0 on a wall),
//   filled row by row from ways[first, first] = 1: O(hw).
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
