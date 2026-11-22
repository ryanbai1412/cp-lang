// expect: ok, output "1 2"
// rule: 04.2.20
// input: "2 3\n...\n..A\n"
let n: Int = read()
let m: Int = read()
domain Row = 0..n
domain Col = 0..m
domain Cell = (Row, Col)
let g: [Cell]Char = read()
let (r, c) = Cell.find(p => g[p] == 'A')!
print(r, c)
