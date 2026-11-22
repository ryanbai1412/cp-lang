// expect: ok, output "B B"
// rule: 04.2.11
// input: "2 3\nA.B\n.B.\n"
let n: Int = read()
let m: Int = read()
domain Row = 0..n
domain Col = 0..m
domain Cell = (Row, Col)
let g: [Cell]Char = read()
let p = Cell((0, 2))
print(g[p], g[1, 1])
