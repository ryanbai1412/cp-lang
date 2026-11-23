// expect: ok, output "[3, 12]\n[3, 4, 5]\n4\n2"
// rule: 04b.1.12
// `g.rows` and `g.cols` are O(1) views of a grid's row and column views, keyed by `Row` and `Col`: `g.rows[r]` is `g[r]` and `g.cols[c]` is `g[.., c]`.
domain Row = 0..2
domain Col = 0..3
let g: [Row, Col]Int = [Row, Col] (r, c) => Int(r) * 3 + Int(c)
print([row.sum() for row in g.rows])
print([col.max()! for col in g.cols])
print(g.cols[1][1])
print(g.cols.len + g.rows.len - 3)
