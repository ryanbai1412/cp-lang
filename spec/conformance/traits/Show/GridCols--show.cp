// expect: ok, output "0 3\n1 4\n2 5"
// rule: std.grid GridCols prints one column per line, in C order
domain R = 0..2
domain C = 0..3
let g: [R, C]Int = [R, C] (r, c) => Int(r) * 3 + Int(c)
print(g.cols)
