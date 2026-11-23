// CF 1970C2 Game on Tree (Medium), t = 1: the stone only moves away from the start,
// so rooted at the start, a vertex wins for the mover iff some child is a losing vertex.

let (n, t): (2..=2e5, 1..=1) = read()
domain V = 1..=n
let es = read<[n - 1](u: V, v: V)>()
let start = read<V>()
let tree = Rooted(Graph.undirected(es), root: start)
// win[v]: 1 if the mover at v wins; a child counts when it is losing (1 - win)
let win = tree.up(+, lift: (w, c, p) => 1 - w, finish: (losing, v) => if losing > 0: 1 else: 0)
print(if win[start] == 1: "Ron" else: "Hermione")
