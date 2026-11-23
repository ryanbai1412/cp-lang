// expect: ok, output "4 B"
// rule: 04.2.11
// input: "2 3\nA.B\n.B.\n"
let (h, w) = read<(1..=10, 1..=10)>()
domain Row = 0..h
domain Col = 0..w
type Cell = (Row, Col)
let grid: [Cell]Char = read()
var free = 0
for p in Cell if grid[p] == '.':
    free += 1
print(free + 1, grid[1, 1])
