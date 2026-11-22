// CCC 2021 S2 Modern Art: each stroke flips a whole row or a whole column of
// a canvas; count the cells flipped an odd number of times.
let (rows, cols, k): (1..=5e6, 1..=5e6, 1..=1e6) = read()
let strokes: [0..k](kind: Char, at: Int) = read()

var row_odd: [0..rows]Bool = false
var col_odd: [0..cols]Bool = false
for (kind, at) in strokes:
    if kind == 'R': row_odd[at - 1] ^= true else: col_odd[at - 1] ^= true

let (r, c) = (count(row_odd), count(col_odd))
// a cell is gold iff exactly one of its row and column is odd
print(r * (cols - c) + c * (rows - r))
