// SHOI 2015 P2, brain surgery: a row of cells, dig out, transplant, longest hole
// original: corpus/submissions/5450923.cpp
let n: 1..=2e5 = read()
let q: 0..=2e5 = read()
domain Cell = 1..=n

struct Run:
    holes: Int = 0
    cells: Int = 0
    best: Int = 0                          // longest stretch of holes
    pre: Int = 0
    suf: Int = 0
    fn join(self, o: Run) -> Run:
        Run(holes + o.holes, cells + o.cells, max(best, o.best, suf + o.pre),
            if cells == 0: holes + o.pre else: pre,
            if o.cells == 0: o.holes + suf else: o.suf)

fn solid(hole: Bool, len: Int) -> Run:
    if hole: Run(len, 0, len, len, len) else: Run(0, len, 0, 0, 0)

// tag: ?Bool, "make this whole stretch holes / tissue"
let paint = Action<Run, ?Bool>(none, (new, old) => new ?? old,
                               (tag, run) => if let hole = tag: solid(hole, run.holes + run.cells) else: run)
var brain = LazySegTree<Cell, Run>(Run(), Run.join, paint, [Cell] _ => solid(false, 1))

for _ in 0..q:
    let kind: 0..=2 = read()
    let (l, r): (Cell, Cell) = read()
    match kind:
        0: brain.apply(l..=r, true)
        2: print(brain.fold(l..=r).best)
        1:
            let (to_l, to_r): (Cell, Cell) = read()
            let moved = brain.fold(l..=r).cells
            if moved > 0:
                brain.apply(l..=r, true)
                // fill holes of to_l..=to_r left to right until the tissue runs out
                let end = brain.max_right(to_l, run => run.holes < moved)
                brain.apply(to_l..=min(end, to_r), false)
