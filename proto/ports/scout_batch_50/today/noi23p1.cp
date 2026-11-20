// NOI 2023 P1, count the lattice cells covered by horizontal, vertical and diagonal strokes (buildable today-form)
// original: corpus/submissions/5723056.cpp
// Strokes are (line, lo, hi) tuples in three lists, and the sweep carries a
// hand-rolled Fenwick over the sorted vertical lines: the prototype has no
// compress(...).first_ge and no Set.rank.
let tc: Int = read()
let rows: 1..=1e9 = read()
let cols: 1..=1e9 = read()
let n: 1..=2e5 = read()
var hori_in: List<(Int, Int, Int)> = []    // (y, x from, x to)
var vert_in: List<(Int, Int, Int)> = []    // (x, y from, y to)
var diag_in: List<(Int, Int, Int)> = []    // (y - x, x from, x to)
for _ in 0..n:
    let kind: 1..=3 = read()
    let x1: Int = read()
    let y1: Int = read()
    let x2: Int = read()
    let y2: Int = read()
    if kind == 1: hori_in.push((y1, min(x1, x2), max(x1, x2)))
    elif kind == 2: vert_in.push((x1, min(y1, y2), max(y1, y2)))
    else: diag_in.push((y1 - x1, min(x1, x2), max(x1, x2)))

// strokes on one line that overlap or touch become one stroke
fn merged(strokes: List<(Int, Int, Int)>) -> List<(Int, Int, Int)>:
    var out: List<(Int, Int, Int)> = []
    for s in sorted(strokes):
        if let top = out.last and top.0 == s.0 and s.1 <= top.2 + 1:
            out[out.len - 1] = (top.0, top.1, max(top.2, s.2))
        else: out.push(s)
    out

let hori = merged(hori_in)
let vert = merged(vert_in)
let diag = merged(diag_in)

var cells = 0
for s in hori: cells += s.2 - s.1 + 1
for s in vert: cells += s.2 - s.1 + 1
for s in diag: cells += s.2 - s.1 + 1

// a cell on a horizontal and a vertical stroke was counted twice: sweep y
// upward keeping the x of every vertical stroke alive at that height
var xs: List<Int> = []
for s in vert: xs.push(s.0)              // already sorted, one per stroke
fn first_ge(x: Int) -> Int:
    var (lo, hi) = (0, xs.len)
    while lo < hi:
        let mid = (lo + hi) / 2
        if xs[mid] < x: lo = mid + 1
        else: hi = mid
    lo
var alive: [0..=xs.len]Int = 0
fn add(at: Int, d: Int):
    var i = at + 1
    while i <= xs.len:
        alive[i] += d
        i += i & -i
fn below(k: Int) -> Int:                 // live strokes among the first k
    var (i, total) = (k, 0)
    while i > 0:
        total += alive[i]
        i -= i & -i
    total

var events: List<(Int, Int, Int, Int)> = []    // (y, order, a, b): leave, enter, then ask
for i in 0..vert.len:
    events.push((vert[i].1, 1, i, 1))
    events.push((vert[i].2 + 1, 0, i, -1))
for s in hori: events.push((s.0, 2, s.1, s.2))
for e in sorted(events):
    if e.1 == 2: cells -= below(first_ge(e.3 + 1)) - below(first_ge(e.2))
    else: add(e.2, e.3)

// a diagonal stroke meets each other stroke in at most one cell
for d in diag:
    var hits: List<Int> = []
    for s in hori:
        let x = s.0 - d.0
        if d.1 <= x and x <= d.2 and s.1 <= x and x <= s.2: hits.push(x)
    for s in vert:
        let y = s.0 + d.0
        if d.1 <= s.0 and s.0 <= d.2 and s.1 <= y and y <= s.2: hits.push(s.0)
    sort(mut hits)
    for i in 0..hits.len:
        if i == 0 or hits[i] != hits[i - 1]: cells -= 1
print(cells)
