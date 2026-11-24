// NOI 2009 "Plants vs. Zombies": zombies come from the right, so a plant shields its left
// neighbour as well as the cells it lists. Plants on or behind a shield cycle can never be eaten;
// among the rest, eating a plant forces eating its shields: a maximum-weight closure.
let (rows, cols): (Int, Int) = read()
domain Row = 0..rows
domain Col = 0..cols
domain Cell = (Row, Col)
let plant = read<[Cell](score: Int, shields: List<Cell>)>()
domain P = 0..Cell.count  // the plants, numbered row by row
let cell = (p: P) => Cell.nth(p.ord)
let at = (c: Cell) => P(c.ord)

fn shielded(p: P) -> List<P>:
    var out = plant[cell(p)].shields.map(at)
    if cell(p).step(Dir4.left) is Some(q): out.push(at(q))
    out
let shields: [P]List<P> = [P] p => shielded(p)
let edible = P.mask(peel(shields))
let eaten = max_closure(
    [P] p => shields[p].filter(q => edible[q]),
    profit: p => if edible[p]: plant[cell(p)].score else: 0,
)
print(eaten.value)
