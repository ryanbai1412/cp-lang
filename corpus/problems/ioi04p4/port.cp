// Source: IOI '04 P4 (Phidias), https://dmoj.ca/problem/ioi04p4
// limits: 1000 ms, 16 MB
// Problem: a W x H slab (W, H <= 600) and n <= 200 wanted plate sizes (no rotation). A cut
//   splits a piece edge to edge, horizontally or vertically, into two integer rectangles;
//   pieces not of a wanted size are waste. Print the least possible waste area.
// Solution: waste(a, b) is 0 if a x b is wanted, else the least of a * b and, over every
//   vertical cut x <= a / 2 and horizontal cut y <= b / 2 (the other halves are symmetric),
//   the waste of its two parts. Filled bottom-up over a <= W, b <= H: O(W H (W + H)).
// Bounds: w, h: 1..=600; n: 1..=200; plates.w: 1..=w; plates.h: 1..=h
let w: 1..=600 = read()
let h: 1..=600 = read()
let n: 1..=200 = read()
let plates: [n](w: 1..=w, h: 1..=h) = read()
let wanted = Set(plates)

// A piece is a plate, or scrap, or it is cut in two; by symmetry the first part of a cut is
// never the larger one, and both parts are smaller, so filling by size in order is enough.
var waste: [1..=w, 1..=h]Int = 0
for a in 1..=w:
    for b in 1..=h:
        if (a, b) in wanted: continue
        let vertical = (1..=a / 2).map(x => waste[x, b] + waste[a - x, b]).min() ?? (a * b)
        let horizontal = (1..=b / 2).map(y => waste[a, y] + waste[a, b - y]).min() ?? (a * b)
        waste[a, b] = min(a * b, vertical, horizontal)
print(waste[w, h])
