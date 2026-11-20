// IOI 1999 Day 2 P3, A Strip of Land
// original: corpus/submissions/4899527.cpp
let (m, n, k): (1..=700, 1..=700, 0..=10) = read()
domain Row = 0..n
domain Col = 0..m
let h: [Row, Col]Int = read()

var best_area = 0
var best: ?(left: Col, right: Col, top: Row, bottom: Row) = none
for left in Col:
    var lo: [Row]Int = [Row] r => h[r, left]
    var hi = lo.copy()
    // FRICTION: "at most 100 columns starting here" leaves the domain for Int and clamps back
    for right in (left.to_idx..left.to_idx + 100).clamp(Col):
        for r in Row:
            lo[r] min= h[r, right]
            hi[r] max= h[r, right]
        // for every top row, the longest run of rows the strip left..=right keeps flat
        let peak = SparseTable<Row, Int>(max, hi)
        let pit = SparseTable<Row, Int>(min, lo)
        for (top, bottom) in slide_until(Row, (t, b) => peak.fold(t..=b) - pit.fold(t..=b) <= k):
            let area = (bottom - top + 1) * (right - left + 1)
            // FRICTION: "first strict maximum, with its witness" is two variables kept in step
            if area > best_area:
                best_area = area
                best = (left, right, top, bottom)

let (left, right, top, bottom) = best!
// the statement numbers rows from the bottom
print(left.to_idx + 1, n - bottom.to_idx, right.to_idx + 1, n - top.to_idx)
