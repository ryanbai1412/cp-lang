// Tuned: the lazy segment tree written out as a recursive tree over two arrays, with 32-bit Run
// fields and an 8-bit paint tag; a query on a painted node answers from the tag without pushing
// it down. A transplant finds where the holes from to_l reach the count in one descent.
//
// Source: SHOI 2015 Brain Hole Therapy Device, https://dmoj.ca/problem/shoi15p2
// limits: 1000 ms, 256 MB
// Problem: n <= 2e5 cells (1-based), all tissue at first, and m <= 2e5 operations: "0 l r" digs out
//   l..=r; "1 l0 r0 l1 r1" digs out l0..=r0 and fills the holes of l1..=r1 left to right with the
//   tissue taken (the rest is thrown away); "2 l r" prints the longest run of holes in l..=r.
// Solution: a lazy SegTree of hole runs (holes, tissue, best, prefix and suffix runs) under an
//   assign-hole-or-tissue action. A transplant counts the tissue in l0..=r0 and digs it out;
//   right_while finds the cell where the holes from l1 on reach that count, and tissue is
//   assigned up to it, or to r1 if it is past r1. O(m log n).
// original: corpus/submissions/5450923.cpp
// Bounds: n, m: 1..=2e5; kind: 0..=2; l, r, to_l, to_r: 1..=n
let (n, m): (1..=2e5, 1..=2e5) = read()
domain Cell = 1..=n

struct Run(holes: 0..=2e5, cells: 0..=2e5, best: 0..=2e5, pre: 0..=2e5, suf: 0..=2e5)

fn solid(hole: Bool, len: 0..=2e5) -> Run = if hole: Run(len, 0, len, len, len) else: Run(0, len, 0, 0, 0)

fn join(a: Run, b: Run) -> Run = Run(
    a.holes + b.holes,
    a.cells + b.cells,
    max(a.best, b.best, a.suf + b.pre),
    if a.cells == 0: a.holes + b.pre else: a.pre,
    if b.cells == 0: b.holes + a.suf else: b.suf,
)

// tag -1: none, 1: all holes, 0: all tissue
var node: List<Run> = List.filled(4 * n, solid(false, 0))
var tag: List<-1..=1> = List.filled(4 * n, -1)

fn build(o: Int, l: Int, r: Int):
    if l == r:
        node[o] = solid(false, 1)
        return
    let mid = (l + r) / 2
    build(2 * o, l, mid)
    build(2 * o + 1, mid + 1, r)
    node[o] = join(node[2 * o], node[2 * o + 1])

fn set(o: Int, len: Int, hole: Bool):
    node[o] = solid(hole, len)
    tag[o] = if hole: 1 else: 0

fn down(o: Int, l: Int, mid: Int, r: Int):
    if tag[o] != -1:
        set(2 * o, mid - l + 1, tag[o] == 1)
        set(2 * o + 1, r - mid, tag[o] == 1)
        tag[o] = -1

fn update(o: Int, l: Int, r: Int, ql: Int, qr: Int, hole: Bool):
    if ql <= l and r <= qr:
        set(o, r - l + 1, hole)
        return
    let mid = (l + r) / 2
    down(o, l, mid, r)
    if ql <= mid: update(2 * o, l, mid, ql, qr, hole)
    if mid < qr: update(2 * o + 1, mid + 1, r, ql, qr, hole)
    node[o] = join(node[2 * o], node[2 * o + 1])

fn query(o: Int, l: Int, r: Int, ql: Int, qr: Int) -> Run:
    if ql <= l and r <= qr: return node[o]
    if tag[o] != -1: return solid(tag[o] == 1, min(r, qr) - max(l, ql) + 1)
    let mid = (l + r) / 2
    if qr <= mid: return query(2 * o, l, mid, ql, qr)
    if mid < ql: return query(2 * o + 1, mid + 1, r, ql, qr)
    return join(query(2 * o, l, mid, ql, qr), query(2 * o + 1, mid + 1, r, ql, qr))

// the first cell from `from` on where the holes reach `need`, or -1
var need = 0
fn reach(o: Int, l: Int, r: Int, from: Int) -> Int:
    if r < from: return -1
    if from <= l and node[o].holes < need:
        need -= node[o].holes
        return -1
    if l == r: return l
    let mid = (l + r) / 2
    down(o, l, mid, r)
    let left = reach(2 * o, l, mid, from)
    if left != -1: return left
    return reach(2 * o + 1, mid + 1, r, from)

build(1, 0, n - 1)
for _ in 0..m:
    let (kind, l, r): (0..=2, Cell, Cell) = read()
    if kind == 0:
        update(1, 0, n - 1, l.ord, r.ord, true)
    elif kind == 2:
        print(query(1, 0, n - 1, l.ord, r.ord).best)
    else:
        let (to_l, to_r): (Cell, Cell) = read()
        let moved = query(1, 0, n - 1, l.ord, r.ord).cells
        if moved == 0: continue
        update(1, 0, n - 1, l.ord, r.ord, true)
        need = moved
        let end = reach(1, 0, n - 1, to_l.ord)
        update(1, 0, n - 1, to_l.ord, if end == -1 or end > to_r.ord: to_r.ord else: end, false)
