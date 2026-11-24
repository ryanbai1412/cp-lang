let n = 200_000
let q = 200_000
domain Cell = 0..n
var seed = 123456789
fn next_rng() -> Int:
    seed = (seed * 1103515245 + 12345) % 2147483648
    seed

struct Run(holes: 0..=200_000, cells: 0..=200_000, best: 0..=200_000, pre: 0..=200_000, suf: 0..=200_000)

fn solid(hole: Bool, len: Int) -> Run = if hole: Run(len, 0, len, len, len) else: Run(0, len, 0, 0, 0)

fn join(a: Run, b: Run) -> Run = Run(
    a.holes + b.holes,
    a.cells + b.cells,
    max(a.best, b.best, a.suf + b.pre),
    if a.cells == 0: a.holes + b.pre else: a.pre,
    if b.cells == 0: b.holes + a.suf else: b.suf,
)

let paint = Action<Run, ?Bool>(
    None,
    (new, old) => if new == None: old else: new,
    (f, v, len) => if let hole = f: solid(hole, len) else: v,
    over: Monoid<Run>(solid(true, 0), join),
    replaces: f => f != None,
)
var brain = SegTree(paint.over, [Cell] _ => solid(false, 1), act: paint)
var acc = 0
for step in 0..q:
    let a = next_rng() % n
    let b = next_rng() % n
    let r = Cell(min(a, b))..=Cell(max(a, b))
    let op = step % 3
    if op == 0:
        brain.apply(r, Some(true))
    elif op == 1:
        brain.apply(r, Some(false))
    else:
        acc += brain.fold(r).best
print(acc)
