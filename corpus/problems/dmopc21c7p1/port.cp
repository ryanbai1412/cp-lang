// Source: DMOPC '21 Contest 7 P1 (Chika Grids), https://dmoj.ca/problem/dmopc21c7p1
// Problem: an N x M grid (N, M <= 1500) of values 0..=1e9, 0 meaning blank. Fill the blanks so
//   every row and every column is strictly increasing with all values in 1..=1e18, keeping
//   the given cells, and print the grid; or print -1 if impossible.
// Solution: give each blank the largest value it can hold: going from the bottom-right, one
//   less than the smaller of the cells below and to its right (1e17 past the border). That
//   leaves the most room for the cells above and to the left, so if this grid fails a final
//   check (positive, at most 1e18, rows and columns increasing) every filling fails. O(NM).
// original: corpus/submissions/4452959.cpp
// Bounds: n, m: 1..=1500; given: 0..=1e9

let n: 1..=1500 = read()
let m: 1..=1500 = read()
domain Row = 1..=n
domain Col = 1..=m
let given: [Row, Col]0..=1e9 = read()
const ROOM = 1e17  // past the border: room below it for n + m blanks
const TOP = 1e18

var a: [Row, Col]Int = 0
for r in Row.rev:
    for c in Col.rev:
        if given[r, c] != 0:
            a[r, c] = given[r, c]
        else:
            var most = ROOM
            if let q = (r, c).down: most min= a[q]
            if let q = (r, c).right: most min= a[q]
            a[r, c] = most - 1

fn below(p: (Row, Col), q: ?(Row, Col)) -> Bool = if let q = q: a[p] < a[q] else: true
let fits = all(
    a[r, c] in 1..=TOP and below((r, c), (r, c).down()) and below((r, c), (r, c).right())
    for r in Row for c in Col
)
if fits: print(a) else: print(-1)
