// Source: IOI 2025 Day 2 (Obstacles for a Llama), https://github.com/ioi/task-archive/tree/master/2025/day2/obstacles
// limits: 2000 ms, 2048 MB
// Problem: an n x m grid, n, m <= 2e5, cell (i, j) free iff t[i] > h[j]. q <= 2e5 queries
//   l r s d: can the llama walk from (0, s) to (0, d) through free cells in columns l..=r?
//   Input as the IOI sample grader gives it.
// Solution: rows with a new maximum temperature split into runs of free columns (layers); an
//   earlier layer drops into the enclosing later run through its columns drier than every row
//   between, keeping the first and last such column. Layers form a forest; binary lifting gives
//   the LCA of the two row-0 layers and the tightest drop on each side: O((n + m + q) log m).
// Bounds: n, m, q: 1..=2e5; t, h: 0..=1e9; l, r, s, d: 0..m
// original: corpus/problems/ioi25_obstacles/orig.cpp
let (n, m) = read<(1..=2e5, 1..=2e5)>()
domain Row = 0..n
domain Col = 0..m
let t: [Row]0..=1e9 = read()
let h: [Col]0..=1e9 = read()

struct Layer(row: Row, cols: Range<Col>)
struct Drop(first: Col, last: Col)

// A path's binding drops: the rightmost `first` and the leftmost `last`.
fn tightest(a: Drop, b: Drop) -> Drop = Drop(max(a.first, b.first), min(a.last, b.last))

let coldest = SparseTable(min, t)
let driest = SparseTable(min, h)
let wettest = SparseTable(max, h)
// The maximal run of columns around `c` drier than `limit`.
fn run_around(c: Col, limit: Int) -> Range<Col>:
    let lo = (Col.first!..=c).first_true(l => wettest.fold(l..=c) < limit)!
    let hi = (c..=Col.last!).last_true(r => wettest.fold(c..=r) < limit)!
    lo..=hi

var layer: [Col]?Layer = None                    // a layer is named by the column that opened it
var up: [Col]?(Col, Drop) = None
var open: [Col]?Col = None                       // layers not yet inside a later run, by first column
var placed: [Col]Bool = false                    // columns already inside a layer
var waiting = Deque.of(Col.all.sorted(by: c => h[c]))   // columns not yet free, driest first
var home: [Col]?Col = None                       // row-0 layer of each free column
for r in t.record_highs():                       // rows with a new maximum temperature
    for id in waiting.pop_front_while(c => h[c] < t[r]):
        if placed[id]: continue
        let run = run_around(id, t[r])
        layer[id] = Some(Layer(r, run))
        // Walk the run, jumping over each open layer inside it; the columns between are new.
        var at = run.first!
        while true:
            var end = at
            if let child = open[at]:
                open[at] = None
                let below = layer[child]!
                let warm = coldest.fold(below.row..=r)
                let cols = below.cols
                end = cols.last!
                if let first = cols.first_true(c => driest.fold(cols.first!..=c) < warm):
                    let last = cols.last_true(c => driest.fold(c..=cols.last!) < warm)!
                    up[child] = Some((id, Drop(first, last)))
            else:
                placed[at] = true
            if end == run.last!: break
            at = Col(end.ord + 1)
        open[run.first!] = id
        if r == Row.first!:
            for c in run: home[c] = Some(id)
let layers = Lift(up, tightest, identity: Drop(Col.first!, Col.last!))

fn can_reach(l: Col, r: Col, s: Col, d: Col) -> Bool:
    let (s, d) = minmax(s, d)
    let (a, b) = (home[s]!, home[d]!)
    let top = layers.lca(a, b) ?? return false
    layers.fold(a, up_to: top).last >= l and layers.fold(b, up_to: top).first <= r

let q: 1..=2e5 = read()
for (l, r, s, d) in read<[q](Col, Col, Col, Col)>():
    print(if can_reach(l, r, s, d): 1 else: 0)
