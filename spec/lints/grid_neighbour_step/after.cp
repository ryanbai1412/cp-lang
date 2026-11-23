type Z = Mod<1_000_000_007>
let (h, w): (2..=1000, 2..=1000) = read()
domain Row = 0..h
domain Col = 0..w
let grid: [Row, Col]Char = read()
var ways: [Row, Col]Z = 0
ways[0, 0] = 1
for (r, c) in (Row, Col):
    if grid[r, c] == '#':
        continue
    if let (pr, pc) = (r, c).step(Dir4.up):
        ways[r, c] += ways[pr, pc]
print(ways[Row(h - 1), Col(w - 1)])
