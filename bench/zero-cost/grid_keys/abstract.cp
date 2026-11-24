// imo22p6checker's walk: every cell of a grid by its keys, reading the cell's row and column.
let n: 1..=1500 = read()
domain Row = 0..n
domain Col = 0..n
domain Value = 1..=n * n
let a: [Row, Col]Value = read()
var s = 0
for _ in 0..20:
    for p in a.keys:
        s += (p.0.label + 1) * a[p].label % (p.1.label + 7)
print(s)
