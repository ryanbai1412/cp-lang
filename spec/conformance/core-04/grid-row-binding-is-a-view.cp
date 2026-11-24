// expect: ok, output "3\n12\n6"
// rule: 04b.1.12 05.3.5
// `let row = g[r]` and `let col = g[.., c]` lend the grid's own cells: no copy, in C++ or anywhere, however long the row (R05.3.5).
domain R = 0..3
domain C = 0..4
let g: [R, C]Int = [R, C] (r, c) => Int(r) * 4 + Int(c)
let row = g[R(1)]
print(row[C(3)] - row[C(0)])
let col = g[.., C(0)]
print(col.sum())
print(row.len + col.len - 1)
