// expect: ok, output "2"
// `bfs` over a named product domain works as over the tuple it names (round 26, B1).
domain R = 0..3
domain C = 0..3
domain Cell = (R, C)
fn nb(p: Cell) -> yields Cell:
    for q in p.adj4: yield q
let ps = bfs(nb, from: [(R(0), C(0))])
print(ps.get((R(2), C(0))) ?? -1)
