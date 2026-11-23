// SHOI 2015 P2, brain surgery: a row of cells, dig out, transplant, longest hole (buildable today-form)
// original: corpus/submissions/5450923.cpp
// The lazy tree is written out by hand: the prototype has no LazySegTree
// with a custom Action and no max_right.
let n: 1..=2e5 = read()
let q: 0..=2e5 = read()

struct Run:
    holes: Int
    cells: Int
    best: Int
    pre: Int
    suf: Int

fn join(a: Run, b: Run) -> Run:
    Run(
        a.holes + b.holes,
        a.cells + b.cells,
        max(max(a.best, b.best), a.suf + b.pre),
        if a.cells == 0: a.holes + b.pre else: a.pre,
        if b.cells == 0: b.holes + a.suf else: b.suf,
    )

fn solid(hole: Bool, len: Int) -> Run:
    if hole: Run(len, 0, len, len, len) else: Run(0, len, 0, 0, 0)

var node: List<Run> = []
var tag: List<Int> = []
for _ in 0..4 * n:
    node.push(solid(false, 1))
    tag.push(-1)

fn build(o: Int, l: Int, r: Int):
    node[o] = solid(false, r - l + 1)
    if l < r:
        let mid = (l + r) / 2
        build(2 * o, l, mid)
        build(2 * o + 1, mid + 1, r)

fn paint(o: Int, l: Int, r: Int, hole: Int):
    node[o] = solid(hole == 1, r - l + 1)
    tag[o] = hole

fn assign(o: Int, l: Int, r: Int, ql: Int, qr: Int, hole: Int):
    if ql <= l and r <= qr:
        paint(o, l, r, hole)
        return
    let mid = (l + r) / 2
    let pending = tag[o]
    if pending != -1:
        paint(2 * o, l, mid, pending)
        paint(2 * o + 1, mid + 1, r, pending)
        tag[o] = -1
    if ql <= mid: assign(2 * o, l, mid, ql, qr, hole)
    if qr > mid: assign(2 * o + 1, mid + 1, r, ql, qr, hole)
    node[o] = join(node[2 * o], node[2 * o + 1])

fn fold(o: Int, l: Int, r: Int, ql: Int, qr: Int) -> Run:
    if ql <= l and r <= qr: return node[o]
    if tag[o] != -1: return solid(tag[o] == 1, min(r, qr) - max(l, ql) + 1)
    let mid = (l + r) / 2
    if qr <= mid: return fold(2 * o, l, mid, ql, qr)
    if ql > mid: return fold(2 * o + 1, mid + 1, r, ql, qr)
    join(fold(2 * o, l, mid, ql, qr), fold(2 * o + 1, mid + 1, r, ql, qr))

build(1, 1, n)
for _ in 0..q:
    let kind: 0..=2 = read()
    let l: Int = read()
    let r: Int = read()
    if kind == 0:
        assign(1, 1, n, l, r, 1)
    elif kind == 2:
        print(fold(1, 1, n, l, r).best)
    else:
        let to_l: Int = read()
        let to_r: Int = read()
        let moved = fold(1, 1, n, l, r).cells
        if moved > 0:
            assign(1, 1, n, l, r, 1)
            // fill holes of to_l..=to_r left to right until the tissue runs out
            let lo = (to_l..to_r).first_true(mid => fold(1, 1, n, to_l, mid).holes >= moved) ?? to_r
            assign(1, 1, n, to_l, lo, 0)
