// CPC '21 Contest 1 P5 (interactive), as it builds today: "the last x that
// still works" has no search, so each one is turned into a first (R1), and
// a reply needs a `let` of its own (R10).
// original: corpus/submissions/4899391.cpp; judge: cpc21c1p5_judge.py
let n: 1..=100 = read()
type Box = (x1: Int, x2: Int, y1: Int, y2: Int)
var found: List<Box> = []

// hidden rectangles inside the box that have not been reported yet
fn left(x1: Int, x2: Int, y1: Int, y2: Int) -> Int:
    let inside: 0..=100 = ask("?", x1, x2, y1, y2)
    inside - count(x1 <= b.x1 and b.x2 <= x2 and y1 <= b.y1 and b.y2 <= y2 for b in found)

// Pin down one rectangle a side at a time: the least right edge, then the
// greatest left edge that goes with it, then the same for bottom and top.
while found.len < n:
    let x2 = (1..=1e9).first_where(x => left(1, x, 1, 1e9) > 0)!
    let x1 = ((1..=x2).first_where(x => left(x, x2, 1, 1e9) == 0) ?? x2 + 1) - 1
    let y2 = (1..=1e9).first_where(y => left(x1, x2, 1, y) > 0)!
    let y1 = ((1..=y2).first_where(y => left(x1, x2, y, y2) == 0) ?? y2 + 1) - 1
    for copy in 0..left(x1, x2, y1, y2): found.push((x1, x2, y1, y2))
print("!")
for b in found: print(b)
