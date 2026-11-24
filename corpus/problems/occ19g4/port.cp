// Source: OCC '19 G4 (Willson and Fighting), https://dmoj.ca/problem/occ19g4
// limits: 1000 ms, 256 MB
// Problem: N <= 60000; 2N geese at distinct points with coordinates in 1..=60000, geese 2k-1
//   and 2k are mates. Goose p honks at goose r if the Manhattan dist(p, r) < dist(p, mate of p);
//   two geese that honk at each other fight once. Print each goose's number of fights.
// Solution: rotate to (x - y, x + y) so Manhattan distance becomes Chebyshev; with radius
//   d = dist to mate - 1, two geese fight iff their distance is <= both radii (mates never
//   are). Sort by radius, largest first. A CDQ divide and conquer over that order gives each
//   goose the larger-radius geese inside its square (sweep the first coordinate, Fenwick over
//   the compressed second); on the reversed order, the smaller-radius squares covering each
//   goose (range add, point query). O(N log^2 N).
// original: corpus/submissions/3578469.cpp
// Bounds: n: 1..=60_000; at.x, at.y: 1..=60_000

let n: 1..=60_000 = read()
domain Goose = 0..2 * n
let at: [Goose](x: 1..=60_000, y: 1..=60_000) = read()

// Rotated to (u, v) = (x - y, x + y), Manhattan distance is Chebyshev.
fn u(g: Goose) -> Int = at[g].x - at[g].y
fn v(g: Goose) -> Int = at[g].x + at[g].y
fn mate(g: Goose) -> Goose = Goose.nth(if g.ord % 2 == 0: g.ord + 1 else: g.ord - 1)
// how far a goose honks; the points are distinct, so it is never negative
let radius: [Goose]Int = [Goose] g => max(abs(u(g) - u(mate(g))), abs(v(g) - v(mate(g)))) - 1

var coords: List<Int> = []
for g in Goose:
    for w in [v(g), v(g) - radius[g], v(g) + radius[g], v(g) + radius[g] + 1]: coords.push(w)
let (domain Y, ys) = compress(coords)
// each goose's second coordinate and its square's edges, compressed
let y = [Goose] g => ys.index_of(v(g))
let lo = [Goose] g => ys.index_of(v(g) - radius[g])
let hi = [Goose] g => ys.index_of(v(g) + radius[g])
let past = [Goose] g => ys.index_of(v(g) + radius[g] + 1)

// largest radius first, ties by number
let order = Goose.sort(by: g => (-radius[g], g.ord))

var bit = Fenwick<Y, Int>()
var ans: [Goose]Int = 0

// each goose's larger-radius geese inside its square: for the right half,
// count the left half's points in the strip u - r..=u + r, as two prefix sweeps
fn cdq_query(l: Int, r: Int):
    if r - l <= 1: return
    let m = (l + r) / 2
    cdq_query(l, m)
    cdq_query(m, r)
    let left = order[l..m].sorted(by: u)
    var edges: List<(u: Int, sgn: Int, g: Goose)> = []
    for g in order[m..r]:
        edges.push((u: u(g) + radius[g], sgn: 1, g:))
        edges.push((u: u(g) - radius[g] - 1, sgn: -1, g:))
    edges.sort(by: e => e.u)
    var li = 0
    for e in edges:
        while li < left.len and u(left[li]) <= e.u:
            bit.add(y[left[li]], 1)
            li += 1
        ans[e.g] += e.sgn * bit.fold(lo[e.g]..=hi[e.g])
    for g in left[..li]: bit.add(y[g], -1)

cdq_query(0, 2 * n)

let rev = order.rev.collect()

// each goose's smaller-radius squares covering it: the left half's squares
// are range adds over their strip, the right half's points are point reads
fn cdq_cover(l: Int, r: Int):
    if r - l <= 1: return
    let m = (l + r) / 2
    cdq_cover(l, m)
    cdq_cover(m, r)
    var edges: List<(u: Int, sgn: Int, g: Goose)> = []
    for g in rev[l..m]:
        edges.push((u: u(g) - radius[g], sgn: 1, g:))
        edges.push((u: u(g) + radius[g] + 1, sgn: -1, g:))
    edges.sort(by: e => e.u)
    let right = rev[m..r].sorted(by: u)
    var li = 0
    for g in right:
        while li < edges.len and edges[li].u <= u(g):
            let e = edges[li]
            bit.add(lo[e.g], e.sgn)
            bit.add(past[e.g], -e.sgn)
            li += 1
        ans[g] += bit.prefix(y[g].r)
    for e in edges[..li]:
        bit.add(lo[e.g], -e.sgn)
        bit.add(past[e.g], e.sgn)

cdq_cover(0, 2 * n)

for g in Goose: print(ans[g])
