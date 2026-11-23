// expect: ok, output "7 -1"
// rule: 04.2.9
// `bfs` over `(R, C)` positions with walls; interpreter and C++ agree (round 26).
domain R = 0..3
domain C = 0..4
let wall: [R, C]Bool = [R, C] (r, c) => Int(c) == 2 and Int(r) < 2
fn nb(p: (R, C)) -> yields (R, C):
    for q in p.adj4:
        if not wall[q]: yield q
let d = bfs(nb, from: [(R(0), C(0))])
print(d.get((R(0), C(3))) ?? -1, d.get((R(1), C(2))) ?? -1)
