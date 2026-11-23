// Source: NOIP 2010 P4, Water Diversion into Cities, https://dmoj.ca/problem/noip10p4
// Problem: an n x m grid of heights (n, m <= 500, heights <= 1e6). Reservoirs can be built in
//   top-row cells; water flows to side-adjacent strictly lower cells. If every bottom-row city
//   can get water print 1 and the fewest reservoirs, else 0 and the number of dry cities.
// Solution: a memoised DFS `flow` gives each cell the bottom-row cities it feeds as [lo, hi];
//   when every city is fed, each top cell's set is an interval. Dry cities are those no top
//   cell reaches; otherwise cover 0..m with the fewest intervals, greedily extending as far
//   right as possible each time. O(n m + m^2).
// original: corpus/submissions/5205786.cpp
let n: 1..=500 = read()
let m: 1..=500 = read()
domain R = 0..n
domain C = 0..m
let h: [R, C]Int = read()

// water runs to a strictly lower side-neighbour. reach[r, c] is the span of
// bottom-row cities fed from (r, c); when every city can be fed it is always
// one interval, so its two ends are all a pump station needs.
var lo: [R, C]Int = m
var hi: [R, C]Int = -1
var seen: [R, C]Bool = false
fn flow(r: R, c: C):
    if seen[r, c]: return
    seen[r, c] = true
    if r == R.last:
        lo[r, c] = c.ord
        hi[r, c] = c.ord
    for (dr, dc) in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
        let rr = r.ord + dr
        let cc = c.ord + dc
        if rr < 0 or rr >= n or cc < 0 or cc >= m: continue
        let (r2, c2) = (R(rr), C(cc))
        if h[r2, c2] < h[r, c]:
            flow(r2, c2)
            lo[r, c] min= lo[r2, c2]
            hi[r, c] max= hi[r2, c2]
for c in C: flow(R.first!, c)

let dry = count(not seen[R.last!, c] for c in C)
if dry > 0:
    print(0)
    print(dry)
else:
    // cover 0..m with the fewest intervals: extend as far right as possible
    var stations = 0
    var covered = 0
    while covered < m:
        var far = covered
        for c in C:
            if lo[R.first!, c] <= covered: far max= hi[R.first!, c] + 1
        covered = far
        stations += 1
    print(1)
    print(stations)
