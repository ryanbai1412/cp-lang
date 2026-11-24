// Source: IOI 2005 Day 1, Garden, https://dmoj.ca/problem/ioi05p1
// limits: 300 ms, 32 MB
// Problem: an l x w garden (l, w <= 250) with n <= 5000 roses on unit squares (several per
//   square allowed). Choose two disjoint axis-parallel rectangles, each holding exactly k roses,
//   with the least total perimeter; print it, or NO.
// Solution: two disjoint rectangles are split by a line between rows or between columns. For
//   each bottom row, `above` grows the top row upward, keeps per-column rose counts of the strip
//   and finds, with a two-pointer window over columns, the narrowest exactly-k rectangles; a
//   running min gives the best one at or above each row. The same on the flipped garden gives
//   the best below; `split` tries every cut, and the transposed garden every vertical cut.
//   O(l w (l + w)).
// original: corpus/submissions/5034025.cpp
// Bounds: length, width: 1..=250; roses: 2..=5000; k: 1..=roses; at.r: 1..=length;
//   at.c: 1..=width
let length: 1..=250 = read()
let width: 1..=250 = read()
let roses: 2..=5000 = read()
let k: 1..=roses = read()
domain Row = 1..=length
domain Col = 1..=width
domain Rose = 1..=roses
let at: [Rose](r: Row, c: Col) = read()
var count: [Row, Col]Int = 0
for (r, c) in at: count[r, c] += 1

// For every row: the least fence around exactly k roses at or above it.
fn above<R, C>(garden: [R, C]Int) -> [R]?Int:
    var best: [R]?Int = None
    var least: ?Int = None
    for bottom in R:
        var strip: [C]Int = 0  // roses per column in rows top..=bottom
        for top in (..=bottom).rev:
            for c in C: strip[c] += garden[top, c]
            var right = 0  // WISH L8: for w in C.stretches(w => sum(strip[w]) >= k)
            var inside = 0
            for left in 0..C.len:
                while right < C.len and inside < k:
                    inside += strip[C.try_at(right)!]
                    right += 1
                if inside == k: least min= 2 * (bottom - top + 1 + right - left)
                inside -= strip[C.try_at(left)!]
        best[bottom] = least
    best

// The two rectangles are split by a line between two rows.
fn split<R, C>(garden: [R, C]Int) -> ?Int:
    let up = above(garden)
    let down = above([R, C] (r, c) => garden[R.try_at(R.len - 1 - r.idx)!, c])  // WISH G2: garden.flipped
    var least: ?Int = None
    for cut in 1..R.len:
        if let a = up[R.try_at(cut - 1)!] and let b = down[R.try_at(R.len - 1 - cut)!]: least min= a + b
    least

let across = split(count)
let along = split([Col, Row] (c, r) => count[r, c])  // WISH L22: count.transposed
if let fence = min(across, along): print(fence)
else: print("NO")
