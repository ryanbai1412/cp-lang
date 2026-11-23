// Source: IOI '04 P4 (Phidias), https://dmoj.ca/problem/ioi04p4
// Problem: a W x H slab (W, H <= 600) and n <= 200 wanted plate sizes (no rotation). A cut
//   splits a piece edge to edge, horizontally or vertically, into two integer rectangles;
//   pieces not of a wanted size are waste. Print the least possible waste area.
// Solution: waste(a, b) is 0 if a x b is wanted, else the least of a * b and, over every
//   vertical cut x <= a / 2 and horizontal cut y <= b / 2 (the other halves are symmetric),
//   the waste of its two parts. Memoised over a <= W, b <= H: O(W H (W + H)).
let w: 1..=600 = read()
let h: 1..=600 = read()
let n: 1..=200 = read()
let plates: [n](w: 1..=w, h: 1..=h) = read()
let wanted = Set(plates)

// A piece is a plate, or scrap, or it is cut in two. By symmetry the first
// part of a cut is never the larger one.
@memo
fn waste(a: 1..=w, b: 1..=h) -> Int:
    if (a, b) in wanted: return 0
    var best = a * b
    for x in 1..=a / 2: best min= waste(x, b) + waste(a - x, b)
    for y in 1..=b / 2: best min= waste(a, y) + waste(a, b - y)
    best
print(waste(w, h))
