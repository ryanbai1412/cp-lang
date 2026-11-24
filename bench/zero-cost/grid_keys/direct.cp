// The same walk as nested loops over the two axes.
let n: 1..=1500 = read()
domain Row = 0..n
domain Col = 0..n
domain Value = 1..=n * n
let a: [Row, Col]Value = read()
var s = 0
for _ in 0..20:
    for r in Row:
        for c in Col:
            s += (r.label + 1) * a[r, c].label % (c.label + 7)
print(s)
