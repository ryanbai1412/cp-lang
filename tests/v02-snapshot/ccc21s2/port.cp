// Source: CCC 2021 S2 (Modern Art), https://dmoj.ca/problem/ccc21s2
// Problem: an M x N canvas (M N <= 5e6) starts black; K <= 1e6 strokes each flip a whole row (R r)
//   or column (C c) between black and gold. Print the number of gold cells.
// Solution: a cell is gold iff its row and column flip counts differ in parity. Keep one parity
//   per row and per column; with r odd rows and c odd columns the gold cells number
//   r (N - c) + c (M - r). O(M + N + K).
let (rows, cols, k): (1..=5e6, 1..=5e6, 1..=1e6) = read()
let strokes: [0..k](kind: Char, at: Int) = read()

var row_odd: [0..rows]Bool = false
var col_odd: [0..cols]Bool = false
for (kind, at) in strokes:
    if kind == 'R': row_odd[at - 1] ^= true else: col_odd[at - 1] ^= true

let (r, c) = (row_odd.count(x => x), col_odd.count(x => x))
// a cell is gold iff exactly one of its row and column is odd
print(r * (cols - c) + c * (rows - r))
