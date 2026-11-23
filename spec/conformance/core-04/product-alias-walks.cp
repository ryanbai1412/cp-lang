// expect: ok, output "3 4"
// rule: 04.2.10
// `type Cell = (Row, Col)` over domains is the product domain (decision 16.5): `for p in Cell` walks it and `grid[p]` indexes it.
domain Row = 1..=2
domain Col = 1..=2
type Cell = (Row, Col)
var grid: [Cell]Char = '.'
grid[Row(1), Col(2)] = '#'
var n = 0
for p in Cell:
    if grid[p] == '.':
        n += 1
print(n, grid.len)
