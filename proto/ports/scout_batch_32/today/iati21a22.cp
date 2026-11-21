// Runnable port (proto/cp.py lib-run) of statement-first iati21a22.cp
// original: corpus/submissions/5053473.cpp
let n: Int = read()
var pts: List<(u: Int, v: Int)> = []
for _ in 0..n:
    let (x, y): (Int, Int) = read()
    pts.push((u: x + y, v: x - y))

// u descending, ties v descending; then the longest strictly increasing run of v.
pts.sort(by: p => (-p.u, -p.v))
var tails: List<Int> = []
for p in pts:
    let lo = tails.first_ge(p.v) ?? tails.len
    if lo == tails.len: tails.push(p.v)
    else: tails[lo] = p.v
print(tails.len)
