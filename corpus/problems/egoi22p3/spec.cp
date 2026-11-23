// EGOI 2022 "Social Engineering": Maria (vertex 1) and an adversary walk a
// graph without reusing edges; whoever cannot move loses. Win if possible.
// Without vertex 1, pair up the friends of 1 by edge-disjoint paths through a
// spanning forest; if one is left unpaired Maria starts there, otherwise she answers
// each adversary move into a friend along its paired path back home.
//
// The task is a grader function (`SocialEngineering`, with the judge's `GetMove` and
// `MakeMove`), and `cpl run` has no stub for a `@grader fn`, so this plays the same game
// over stdin: it reads the sample grader's `n m` and edges, asks `?` for Maria's move
// (0 once she resigns), prints each of its moves, and prints `!` where the function returns.
let wire = Wire()
let (n, m) = read<(2..=2e5, 1..=4e5)>()
domain V = 1..=n
let edges = read<[m](u: V, v: V)>()
let g = Graph.undirected(edges)
let home = V.first!
let friend = V.mask(g[home])
let rest = g.only(v => v != home)
var parts = Dsu<V>()
for e in edges if e.u != home and e.v != home: parts.unite(e.u, e.v)
// One tree per component of `rest`; `home` has no arcs there, so it is a tree of its own.
let forest = Rooted.forest(rest, from: [home] + g[home].distinct(by: v => parts.find(v)))

// Children before parents: each vertex pairs the friends left open below it through
// itself and passes at most one up.
var open_below: [V]?V = None
var route: [V]List<V> = []
for u in forest.order.rev:
    let open = List(chain(if friend[u]: [u] else: [], forest.children(u).filter_map(c => open_below[c])))
    for pair in open.chunks(2) if pair.len == 2:
        let path = forest.path(pair.first!, pair.last!)
        route[path.first!] = path
        route[path.last!] = path.rev.to_list()
    if open.len % 2 == 1: open_below[u] = open.last

// A friend left open in some tree: Maria wins from there, so resign at once.
if forest.roots.all(r => open_below[r] == None):
    while let u = V.at(wire.ask("?")):
        for v in route[u].drop(1): print(v)
        print(home)
print("!")
