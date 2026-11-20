// cp: 0.1 1f59ac0+dirty
// AtCoder DP H Grid 1: count the paths from the top-left to the bottom-right
// corner that move right or down through open squares.
const P = 1000000007
type Z = Mod<P>
let h: 2..=1000 = read()
let w: 2..=1000 = read()
domain Row = 1..=h
domain Col = 1..=w
fn random_grid() -> [Row, Col]Char:
    [Row, Col] (r, c) => if rand(0, 4) == 0 and (r, c) != (Row.first, Col.first) and (r, c) != (Row.last, Col.last): '#' else: '.'
let grid: [Row, Col]Char = read(gen: random_grid) where grid[Row.first, Col.first] == '.' and grid[Row.last, Col.last] == '.'

var ways: [Row, Col]Z = Z(0)
ways[Row.first, Col.first] = Z(1)
for r in Row:
    for c in Col:
        if grid[r, c] == '.':
            if let up = r.prev: ways[r, c] += ways[up, c]
            if let left = c.prev: ways[r, c] += ways[r, left]
print(ways[Row.last, Col.last])
