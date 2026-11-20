// `@memo` over a product of domains too large for a table stores only the points asked for.
domain Row = 0..6311529
domain Col = 0..432463
var calls = 0
fn slow(r: Row, c: Col) -> Int:
    r.idx * 7 + c.idx
@memo
fn cost(r: Row, c: Col) -> Int:
    slow(r, c)
print(cost(Row(5), Col(9)), cost(Row(5), Col(9)), cost(Row(6311528), Col(432462)))
