// A named product position is a plain vertex, its own `Target::to`, so
// `bfs` over `domain Cell = (R, C)` works as over the tuple it names.
domain R = 0..3
domain C = 0..3
domain Cell = (R, C)
fn nb(p: Cell) -> yields Cell:
    for q in p.adj4: yield q
let ps = bfs(nb, from: [(R(0), C(0))])
print(ps.get((R(2), C(0))) ?? -1, ps.get((R(2), C(2))) ?? -1)
