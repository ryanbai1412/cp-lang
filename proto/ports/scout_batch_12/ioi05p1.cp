// IOI 2005 P1, Garden: roses stand on a grid. Fence two rectangles that
// do not overlap, each holding exactly k roses, with the smallest total
// length of fence. Print NO if there are no two such rectangles.
// original: corpus/submissions/5034025.cpp
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
    var best: [R]?Int = none
    var least: ?Int = none
    for bottom in R:
        var strip: [C]Int = 0                             // roses per column in rows top..=bottom
        for top in (..=bottom).rev:
            for c in C: strip[c] += garden[top, c]
            var right = 0                                 // WISH L8: for w in C.stretches(w => sum(strip[w]) >= k)
            var inside = 0
            for left in 0..C.len:
                while right < C.len and inside < k:
                    inside += strip[C.from_idx(right)]
                    right += 1
                if inside == k: least min= 2 * (bottom - top + 1 + right - left)
                inside -= strip[C.from_idx(left)]
        best[bottom] = least
    best

// The two rectangles are split by a line between two rows.
fn split<R, C>(garden: [R, C]Int) -> ?Int:
    let up = above(garden)
    let down = above([R, C] (r, c) => garden[R.from_idx(R.len - 1 - r.idx), c])   // WISH G2: garden.flipped
    var least: ?Int = none
    for cut in 1..R.len:
        if let a = up[R.from_idx(cut - 1)] and let b = down[R.from_idx(R.len - 1 - cut)]: least min= a + b
    least

let across = split(count)
let along = split([Col, Row] (c, r) => count[r, c])       // WISH L22: count.transposed
if let fence = min(across, along): print(fence)
else: print("NO")
