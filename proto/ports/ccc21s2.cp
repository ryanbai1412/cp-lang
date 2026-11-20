// cp: 0.1 8ef64b9+dirty
// CCC 2021 S2 Modern Art: each stroke flips a whole row or column;
// count cells flipped an odd number of times.
let rows: 1..=5e6 = read()
let cols: 1..=5e6 = read() where rows * cols <= 5e6
let k: 1..=1e6 = read()
domain Row = 1..=rows
domain Col = 1..=cols

@tags("R", "C")
enum Stroke:
    Across(r: Row)
    Down(c: Col)

let strokes: [k]Stroke = read()

var row_odd: [Row]Bool = false
var col_odd: [Col]Bool = false
for s in strokes:
    match s:
        Across(r) =>
            row_odd[r] = not row_odd[r]
        Down(c) =>
            col_odd[c] = not col_odd[c]

let odd_rows = count(row_odd[r] for r in Row)
let odd_cols = count(col_odd[c] for c in Col)
// a cell is gold iff exactly one of its row and column is odd
print(odd_rows * (Col.len - odd_cols) + odd_cols * (Row.len - odd_rows))
