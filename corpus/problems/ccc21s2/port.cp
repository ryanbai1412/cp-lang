// Source: CCC 2021 S2 (Modern Art), https://dmoj.ca/problem/ccc21s2
// limits: 3000 ms, 1024 MB
// Problem: an M x N canvas (M N <= 5e6) starts black; K <= 1e6 strokes each flip a whole row (R r)
//   or column (C c) between black and gold. Print the number of gold cells.
// Solution: a cell is gold iff its row and column flip counts differ in parity. Keep one parity
//   per row and per column; with r odd rows and c odd columns the gold cells number
//   r (N - c) + c (M - r). O(M + N + K).
// original: corpus/submissions/4323443.cpp
// Bounds: rows, cols: 1..=5e6; k: 1..=1e6
let rows: 1..=5e6 = read()
let cols: 1..=5e6 = read()
let k: 1..=1e6 = read()
domain Row = 1..=rows
domain Col = 1..=cols
var row_odd: [Row]Bool = false
var col_odd: [Col]Bool = false
repeat k:
    let kind: Char = read()
    let at: Int = read()
    if kind == 'R': row_odd[Row(at)] ^= true else: col_odd[Col(at)] ^= true
let r = row_odd.count(odd => odd)
let c = col_odd.count(odd => odd)
print(r * (cols - c) + c * (rows - r))
