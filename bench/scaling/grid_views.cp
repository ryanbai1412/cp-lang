// The rows of an n by 64 grid and the columns of a 64 by n one, each row or column one O(1) view.
let n: Int = read()
domain R = 0..n
domain K = 0..64
domain C = 0..n
let tall: [R, K]Int = 1
let wide: [K, C]Int = 1
var s = 0
for i in 0..64 * n:
    s += tall.rows.at(R(i * 7919 % n))[K(i % 64)] // timed: grid.GridRows.at
    s += wide.cols.at(C(i * 7919 % n))[K(i % 64)] // timed: grid.GridCols.at
for r in 0..8192:
    for row in tall.rows.each(): // timed: grid.GridRows.each
        s += row[K(r % 64)]
    for col in wide.cols.each(): // timed: grid.GridCols.each
        s += col[K(r % 64)]
print(s)
