// expect: ok, output "20 2 22 23\n13 12 11 10\n3 21 1 0\n4 5 3 1 2"
// rule: 04b.1.12
// `g.rows[r]` and `g.cols[c]` are places when written through, as `g[r]` is: a row can be sorted, reversed or swapped with another in place, and two slices of one length swap their cells.
domain Row = 0..3
domain Col = 0..4
var g: [Row, Col]Int = [Row, Col] (r, c) => 10 * Int(r) + 3 - Int(c)
let r1 = Row.first!
let r2 = Row.last!
if r1 != r2:
    swap(mut g.rows[r1], mut g.rows[r2])
g.rows[r1].sort()
reverse(mut g.cols[Col(1)])
for r in Row:
    print(g.rows[r])
var a = [1, 2, 3, 4, 5]
swap(mut a[0..2], mut a[3..5])
print(a)
