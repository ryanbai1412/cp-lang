// Source: OCC '19 G4 (Willson and Fighting), https://dmoj.ca/problem/occ19g4
// limits: 1000 ms, 256 MB
// Problem: N <= 60000; 2N geese at distinct points with coordinates in 1..=60000, geese 2k-1
//   and 2k are mates. Goose p honks at goose r if the Manhattan dist(p, r) < dist(p, mate of p);
//   two geese that honk at each other fight once. Print each goose's number of fights.
// Solution: rotate to (u, v) = (x - y, x + y) so Manhattan distance becomes Chebyshev; with
//   radius d = dist to mate - 1, two geese fight iff their distance is <= both radii (mates
//   never are), i.e. the smaller-radius goose's square holds the other. Rank by radius, largest
//   first, and divide and conquer once over the ranks: across a split, each later goose counts
//   the earlier geese in its square (a point sweep over u, Fenwick over v), and each earlier
//   goose counts the later squares holding it (a range-add sweep). The three u orders are
//   sorted once and split stably down the recursion. O(N log^2 N).
// original: corpus/problems/occ19g4/orig.cpp (brute force; no saved submission is accepted)
// Bounds: n: 1..=60_000; at.x, at.y: 1..=60_000

let n: 1..=60_000 = read()
domain Goose = 0..2 * n
let at: [Goose](x: 1..=60_000, y: 1..=60_000) = read()

// Rotated, Manhattan distance is Chebyshev; v = x + y is at most top.
let top = 2 * 60_000
domain Y = 0..=top + 1
let u: [Goose]Int = [Goose] g => at[g].x - at[g].y
let v: [Goose]Int = [Goose] g => at[g].x + at[g].y
fn mate(g: Goose) -> Goose = Goose.nth(if g.ord % 2 == 0: g.ord + 1 else: g.ord - 1)
// how far a goose honks; the points are distinct, so it is never negative
let radius: [Goose]Int = [Goose] g => max(abs(u[g] - u[mate(g)]), abs(v[g] - v[mate(g)])) - 1
// each goose's own v, its square's v edges (clamped to where geese are) and one past the top
let y: [Goose]Y = [Goose] g => Y(v[g])
let lo: [Goose]Y = [Goose] g => Y(max(v[g] - radius[g], 0))
let past: [Goose]Y = [Goose] g => Y(min(v[g] + radius[g], top) + 1)
let (start, stop) = ([Goose] g => u[g] - radius[g], [Goose] g => u[g] + radius[g])

// largest radius first, ties by number
let rank = Goose.sort(by: g => (-radius[g], g.ord)).inverse()

var bit = Fenwick<Y, Int>()
var ans: [Goose]Int = 0

// Each later goose's count of earlier points with u <= its key, inside its v strip.
fn points_below(earlier: List<Goose>, later: List<Goose>, key: Goose -> Int, sgn: Int):
    var i = 0
    for g in later:
        while i < earlier.len and u[earlier[i]] <= key(g):
            bit.add(y[earlier[i]], 1)
            i += 1
        ans[g] += sgn * bit.fold(lo[g]..past[g])
    for p in earlier[..i]: bit.add(y[p], -1)

// Each point's count of squares that open at u <= its own u and hold its v.
fn squares_below(squares: List<Goose>, opens: Goose -> Int, points: List<Goose>, sgn: Int):
    var i = 0
    for g in points:
        while i < squares.len and opens(squares[i]) <= u[g]:
            bit.add(lo[squares[i]], 1)
            bit.add(past[squares[i]], -1)
            i += 1
        ans[g] += sgn * bit.prefix(y[g].r)
    for p in squares[..i]:
        bit.add(lo[p], -1)
        bit.add(past[p], 1)

// by_u, by_start, by_stop: the geese ranked l..r, each sorted by that key
fn split(l: Int, r: Int, by_u: List<Goose>, by_start: List<Goose>, by_stop: List<Goose>):
    if r - l <= 1: return
    let m = (l + r) / 2
    let (eu, lu) = by_u.partition(g => rank[g].ord < m)
    let (es, ls) = by_start.partition(g => rank[g].ord < m)
    let (et, lt) = by_stop.partition(g => rank[g].ord < m)
    // a later goose's square holds an earlier point: u in start..=stop
    points_below(eu, lt, g => stop[g], 1)
    points_below(eu, ls, g => start[g] - 1, -1)
    // and that earlier goose counts the later square: start <= u < stop + 1
    squares_below(ls, g => start[g], eu, 1)
    squares_below(lt, g => stop[g] + 1, eu, -1)
    split(l, m, eu, es, et)
    split(m, r, lu, ls, lt)

split(
    0,
    2 * n,
    Goose.sort(by: g => u[g]),
    Goose.sort(by: g => start[g]),
    Goose.sort(by: g => stop[g]),
)

for g in Goose: print(ans[g])
