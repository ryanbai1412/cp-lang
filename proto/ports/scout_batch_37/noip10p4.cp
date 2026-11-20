// NOIP 2010 P4, Water Diversion into Cities
// original: corpus/submissions/5205786.cpp
let n: 1..=500 = read()
let m: 1..=500 = read()
domain R = 1..=n
domain C = 1..=m
type Cell = (R, C)
let h: [R, C]Int = read()

// water runs to a strictly lower side-neighbour. fed[p] is the span of
// bottom-row cities fed from p; when every city can be fed it is always one
// interval, so its two ends are all a pump station needs.
@memo
fn fed(p: Cell) -> (lo: Int, hi: Int):
    let (r, c) = p
    var span = if r == R.last: (lo: c.to_idx, hi: c.to_idx + 1) else: (lo: m, hi: 0)
    for q in p.adj4 if h[q] < h[p]:
        span.lo min= fed(q).lo
        span.hi max= fed(q).hi
    return span

// one flood from the whole top row decides which cities stay dry
var wet: [Cell]Bool = false
fn flood(p: Cell):
    wet[p] = true
    for q in p.adj4 if h[q] < h[p] and not wet[q]: flood(q)
for c in C if not wet[(R.first, c)]: flood((R.first, c))

let dry = count(not wet[(R.last, c)] for c in C)
if dry > 0:
    print(0)
    print(dry)
else:
    // cover 0..m with the fewest intervals: extend as far right as possible
    var (stations, covered) = (0, 0)
    while covered < m:
        covered = max(fed((R.first, c)).hi for c in C if fed((R.first, c)).lo <= covered)!
        stations += 1
    print(1)
    print(stations)
