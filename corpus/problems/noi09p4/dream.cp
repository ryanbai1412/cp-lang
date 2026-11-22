let (rows, cols) = read<(1..=20, 1..=30)>()
domain Cell = ([rows], [cols])
let plant = read<[Cell](score: -1e4..=1e4, protects: List<Cell>)>()
fn protects(p: Cell) = chain(plant[p].protects, p.step(Dir4.left))
let edible = Cell.mask(peel(protects))
let eaten = max_closure(
    Cell,
    before: p => protects(p).filter(q => edible[q]),
    profit: p => if edible[p]: plant[p].score else: 0,
)
print(eaten.value)
