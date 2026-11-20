// Runnable port (proto/cp.py lib-run) of statement-first ioi04p2hard.cp
// original: corpus/submissions/4182973.cpp
let n: Int = read()
let BIG = 1000000000000000000
// Point i is (a[i], b[i]); point 0 is the start (0, 0).
var a = [0]
var b = [0]
for _ in 0..n:
    let (x, y): (Int, Int) = read()
    a.push(x)
    b.push(y)
var walk_a = [0]
var walk_b = [0]
for i in 0..n:
    walk_a.push(walk_a[i] + abs(a[i] - a[i + 1]))
    walk_b.push(walk_b[i] + abs(b[i] - b[i + 1]))

// Four "minimum value over keys <= x" tables, each a list sorted by key with
// strictly useful entries only (a later key is kept only if its value is smaller).
var keys: List<List<Int>> = [[], [], [], []]
var vals: List<List<Int>> = [[], [], [], []]

fn insert(t: Int, k: Int, v: Int):
    var at = 0
    while at < keys[t].len and keys[t][at] <= k: at += 1
    if at > 0 and vals[t][at - 1] <= v: return
    var nk: List<Int> = []
    var nv: List<Int> = []
    for i in 0..at:
        if keys[t][i] != k:
            nk.push(keys[t][i])
            nv.push(vals[t][i])
    nk.push(k)
    nv.push(v)
    var rest = at
    while rest < keys[t].len and vals[t][rest] > v: rest += 1
    for i in rest..keys[t].len:
        nk.push(keys[t][i])
        nv.push(vals[t][i])
    keys[t] = nk
    vals[t] = nv

fn upto(t: Int, x: Int) -> Int:
    var best = BIG
    for i in 0..keys[t].len:
        if keys[t][i] <= x: best = vals[t][i]
    return best

// Tables 0/1 hold offers keyed by a y coordinate, tables 2/3 by an x coordinate.
// Even = keyed by the coordinate (for queries from above), odd = by its negation.
fn offer_a(i: Int, cost: Int):
    if i == 0: return
    let base = walk_a[i - 1] + cost
    insert(0, b[i], base - b[i])
    insert(1, -b[i], base + b[i])

fn offer_b(i: Int, cost: Int):
    if i == 0: return
    let base = walk_b[i - 1] + cost
    insert(2, a[i], base - a[i])
    insert(3, -a[i], base + a[i])

var (da, db) = (0, 0)
offer_a(n, 0)
offer_b(n, 0)
for i in (0..n).rev:
    let pa = if i > 0: a[i - 1] else: 0
    da = min(upto(2, pa) + pa, upto(3, -pa) - pa)
    da = min(da - walk_b[i], walk_b[n] - walk_b[i])
    let pb = if i > 0: b[i - 1] else: 0
    db = min(upto(0, pb) + pb, upto(1, -pb) - pb)
    db = min(db - walk_a[i], walk_a[n] - walk_a[i])
    offer_a(i, da)
    offer_b(i, db)
print(min(da, db))
