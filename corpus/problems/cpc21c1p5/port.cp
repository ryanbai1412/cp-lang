// Source: CPC '21 Contest 1 P5 (AQT and Modern Art), https://dmoj.ca/problem/cpc21c1p5
// Problem: interactive. N <= 500 hidden rectangles [x1, x2] x [y1, y2], coordinates in
//   1..1e9, duplicates possible. "? a b c d" returns how many lie fully inside
//   [a, b] x [c, d]; at most 61000 queries; then print "!" and all N rectangles.
// Solution: left(box) is the reply minus the rectangles already found inside it. Binary
//   search the least x2 with an unfound rectangle in [1, x2] x all, then the greatest x1
//   with one in [x1, x2], then the least y2 and greatest y1 likewise: that pins one
//   rectangle exactly, and one more query gives its multiplicity. About 4 * 30 + 1 queries
//   per distinct rectangle.
// original: corpus/submissions/4899391.cpp; interactor: tests/interactor.py
// Bounds: n: 1..=500
const SIDE = 1e9  // coordinates are in 1..=SIDE
let n: 1..=500 = read()
type Box = (x1: Int, x2: Int, y1: Int, y2: Int)
var found: List<Box> = []

// hidden rectangles inside the box that have not been reported yet
fn left(x1: Int, x2: Int, y1: Int, y2: Int) -> Int:
    let inside: 0..=500 = ask("?", x1, x2, y1, y2)
    inside - count(x1 <= b.x1 and b.x2 <= x2 and y1 <= b.y1 and b.y2 <= y2 for b in found)

// Pin down one rectangle a side at a time: the least right edge, then the
// greatest left edge that goes with it, then the same for bottom and top.
while found.len < n:
    let x2 = (1..=SIDE).first_true(x => left(1, x, 1, SIDE) > 0)!
    let x1 = (1..=x2).last_true(x => left(x, x2, 1, SIDE) > 0)!
    let y2 = (1..=SIDE).first_true(y => left(x1, x2, 1, y) > 0)!
    let y1 = (1..=y2).last_true(y => left(x1, x2, y, y2) > 0)!
    for _ in 0..left(x1, x2, y1, y2): found.push((x1, x2, y1, y2))
print("!")
for b in found: print(b)
