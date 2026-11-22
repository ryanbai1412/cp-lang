// noi09p4 Plants vs Zombies: a plant can be eaten only after every plant guarding it, and each
// plant guards the one to its left. Plants on a guard cycle, or guarded from one, never fall;
// peeling removes exactly the others. Among those, eating a set closed under "guarded by" is a
// maximum-weight closure.
let (rows, cols) = read<(1..=20, 1..=30)>()
domain Cell = ([rows], [cols])
let plant = read<[Cell](score: -1e4..=1e4, protects: List<Cell>)>()
fn protects(p: Cell) = chain(plant[p].protects, p.step(Dir4.left))
let edible = Cell.mask(peel::<Cell>(protects))
let eaten = max_closure(
    before: p => protects(p).filter(q => edible[q]),
    profit: p => if edible[p]: plant[p].score else: 0,
)
print(eaten.value)
