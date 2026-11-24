// expect: ok, output "3"
// rule: 05.2.1 05.3.5
// An Int read through two index levels holds no lend, so a struct stored past its block can keep it.
struct Cand(at: Int, v: Int)
let adj: List<List<Int>> = [[1, 2], [3]]
var best: List<Cand> = []
for u in 0..2:
    let v = adj[u][0]
    best.push(Cand(u, v))
print(best[1].v)
