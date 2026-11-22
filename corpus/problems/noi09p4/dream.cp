// noi09p4 Plants vs Zombies: a plant can be eaten only after every plant guarding it, and each
// plant guards the one to its left. Plants on a guard cycle, or guarded from one, never fall;
// peeling removes exactly the others. Among those, eating a set closed under "guarded by" is a
// maximum-weight closure.
let (rows, cols) = read<(1..=20, 1..=30)>()
domain Row = [rows]
domain Col = [cols]
domain Cell = (Row, Col)
var score: [Cell]Int = 0
var guards: [Cell]List<Cell> = []
for p in Cell:
    let (s, k) = read<(-10_000..=10_000, 0..=600)>()
    score[p] = s
    guards[p] = List(read<[k](Row, Col)>())
    if let left = p.step(Dir4.left()):
        guards[p].push(left)
var falls: [Cell]Bool = false
for p in peel::<Cell>(p => guards[p]):
    falls[p] = true
let needs = [(u: p, v: q) for p in Cell if falls[p] for q in guards[p] if falls[q]]
print(max_closure([Cell] p => if falls[p]: score[p] else: 0, needs).value)
