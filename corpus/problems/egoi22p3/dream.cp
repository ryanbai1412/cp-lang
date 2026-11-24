// EGOI 2022 "Social Engineering": Maria (vertex 1) and an adversary walk a graph without reusing
// edges; whoever cannot move loses. Away from home the friends (home's neighbours) must pair up
// along disjoint paths of a DFS forest; then Maria answers every move with the rest of its path.
let wire = Wire()
let (n, m) = read<(2..=2e5, 1..=4e5)>()
domain V = 1..=n
let g = Graph.undirected(read<[m](u: V, v: V)>())
let home = V.first!
var friend: [V]Bool = false
for v in g.next(home): friend[v] = true
let forest = Rooted.forest(g.only(keep: v => v != home, from: v => v != home))
var route: [V]List<V> = []

fn pair_up(u: V) -> ?V:  // the friend under u left open, if any
    let open = chain(if friend[u]: [u] else: [], forest.children(u).filter_map(pair_up)).collect()
    for pair in open.chunks(2):
        let (a, b) = (pair.first!, pair.last!)
        if a != b:
            route[a] = forest.path(a, b)
            route[b] = forest.path(b, a)
    if open.len % 2 == 1: open.last else: None

if forest.roots.all(r => r == home or pair_up(r) == None):
    while true:
        let move: 0..=n = wire.ask("?")
        if move == 0: break
        for v in route[V(move)].drop(1): print(v)
        print(home)
print("!")
