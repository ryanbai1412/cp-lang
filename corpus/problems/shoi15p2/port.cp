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

struct Run(holes: Int, cells: Int, best: Int, pre: Int, suf: Int)

fn solid(hole: Bool, len: Int) -> Run = if hole: Run(len, 0, len, len, len) else: Run(0, len, 0, 0, 0)

fn join(a: Run, b: Run) -> Run = Run(
    a.holes + b.holes,
    a.cells + b.cells,
    max(a.best, b.best, a.suf + b.pre),
    if a.cells == 0: a.holes + b.pre else: a.pre,
    if b.cells == 0: b.holes + a.suf else: b.suf,
)

// a tag paints its span all holes (true) or all tissue (false)
let paint = Action<Run, ?Bool>(
    None,
    (new, old) => if new == None: old else: new,
    (f, v, len) => if let hole = f: solid(hole, len) else: v,
    over: Monoid<Run>(solid(true, 0), join),
)
var brain = SegTree(paint.over, [Cell] _ => solid(false, 1), act: paint)

for _ in 0..m:
    let (kind, l, r): (0..=2, Cell, Cell) = read()
    if kind == 0:
        brain.apply(l..=r, Some(true))
    elif kind == 2:
        print(brain.fold(l..=r).best)
    else:
        let (to_l, to_r): (Cell, Cell) = read()
        let moved = brain.fold(l..=r).cells
        if moved == 0: continue
        brain.apply(l..=r, Some(true))
        let short = brain.right_while(to_l.l, s => s.holes < moved)
        brain.apply(to_l..=Cell.nth(min(short.ord, to_r.ord)), Some(false))
