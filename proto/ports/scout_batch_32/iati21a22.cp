// IATI '21 A2 P2 (DMOJ: iati21a22) - longest chain of diagonal moves
// Ideal v2 port. original: corpus/submissions/5053473.cpp
//
// Points on the plane; from (x, y) one may step to a point whose rotated
// coordinates u = x + y and v = x - y satisfy u' <= u and v' > v. Print the
// longest such chain: sort by u descending and take a strictly increasing
// subsequence of v (ties in u go v-descending so they cannot chain).
//
// The C++ compresses both axes first; here only the order matters, so
// `compress` is not needed at all.
let n: 1..=2e5 = read()
let pts: [0..n](x: -1e9..=1e9, y: -1e9..=1e9) = read()

let vs = sorted((u: p.x + p.y, v: p.x - p.y) for p in pts, by: p => (desc(p.u), desc(p.v)))
var tails: List<Int> = []
for (_, v) in vs:
    // WISH X3: "replace the first element >= v, else append" is LIS's whole
    // inner step; `first_ge` makes it two lines, a `lis(vs, strict: true)` none.
    if let i = tails.first_ge(v): tails[i] = v
    else: tails.push(v)
print(tails.len)
