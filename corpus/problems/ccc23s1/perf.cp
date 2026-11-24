// Tuned: one pass per row counting tiles and touching pairs with the previous cell.
//
// Source: CCC 2023 S1 (Trianglane), https://dmoj.ca/problem/ccc23s1
// Problem: two rows of c <= 2e5 unit triangle tiles (the first points up, neighbours alternate),
//   each black (1) or white (0). Print the length of tape around the black area: the black
//   tiles' sides not shared with another black tile.
// Solution: each black tile has 3 sides, and each side two black tiles share takes 2 off the
//   tape: neighbours in a row, and at even columns the two tiles across. O(c).
// original: corpus/submissions/5350804.cpp
// Bounds: n: 1..=2e5; black: 0..=1
let n: 1..=2e5 = read()
domain Row = 0..2
domain Col = 0..n
let black: [Row, Col]0..=1 = read()
var (tiles, sideways) = (0, 0)
for r in Row:
    var prev = 0
    for c in Col:
        let b = black[r, c]
        tiles += b
        sideways += prev * b
        prev = b
let across = Col.count(c => c.ord % 2 == 0 and black[Row.first!, c] + black[Row.last!, c] == 2)
print(3 * tiles - 2 * (sideways + across))
