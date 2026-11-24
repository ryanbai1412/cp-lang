// Source: NOIP 2010 P4, Water Diversion into Cities, https://dmoj.ca/problem/noip10p4
// Problem: an n x m grid of heights (n, m <= 500, heights <= 1e6). Reservoirs can be built in
//   top-row cells; water flows to side-adjacent strictly lower cells. If every bottom-row city
//   can get water print 1 and the fewest reservoirs, else 0 and the number of dry cities.
// Solution: a memoised DFS `flow` gives each cell the span of bottom-row cities it feeds;
//   when every city is fed, each top cell's set is an interval. Dry cities are those no top
//   cell reaches; otherwise cover 0..m with the fewest intervals, greedily extending as far
//   right as possible each time. O(n m + m^2).
// original: corpus/submissions/5205786.cpp
use std.grid
let n: 1..=500 = read()
let m: 1..=500 = read()
domain R = 0..n
domain C = 0..m
let h: [R, C]0..=1_000_000 = read()

// water runs to a strictly lower side-neighbour. span[r, c] is the range of bottom-row cities
// fed from (r, c); when every city can be fed it is always one interval, so its two ends are
// all a reservoir needs
type Span = (lo: C, hi: C)
fn widen(s: ?Span, t: Span) -> Span:
    if let (lo, hi) = s: (min(lo, t.lo), max(hi, t.hi)) else: t
var span: [R, C]?Span = None
var seen: [R, C]Bool = false
fn flow(r: R, c: C):
    if seen[r, c]: return
    seen[r, c] = true
    if r == R.last!: span[r, c] = Some((c, c))
    for d in Dir4.all():
        if let (r2, c2) = (r, c).step(d) and h[r2, c2] < h[r, c]:
            flow(r2, c2)
            if let t = span[r2, c2]: span[r, c] = Some(widen(span[r, c], t))
for c in C: flow(R.first!, c)

let dry = count(not seen[R.last!, c] for c in C)
if dry > 0:
    print(0)
    print(dry)
else:
    // cover the cities with the fewest spans: extend as far right as possible each time
    var stations = 0
    var covered = 0  // cities 0..covered are fed
    while covered < m:
        var far = covered
        for c in C:
            if let t = span[R.first!, c] and t.lo.ord <= covered: far max= t.hi.ord + 1
        covered = far
        stations += 1
    print(1)
    print(stations)
