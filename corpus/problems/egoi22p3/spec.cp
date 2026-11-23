// EGOI 2022 "Social Engineering": Maria (vertex 1) and an adversary walk a
// graph without reusing edges; whoever cannot move loses. Win if possible.
// Without vertex 1, pair up the friends of 1 by edge-disjoint paths through a
// DFS forest; if one is left unpaired Maria starts there, otherwise she answers
// each adversary move into a friend along its paired path back home.
extern fn get_move() -> Int
extern fn make_move(v: Int)

@grader
fn social_engineering(n: Int, edges: List<(Int, Int)>):
    domain V = 1..=n
    let g = Graph.undirected([edges.keys] e => (u: V(edges[e].0), v: V(edges[e].1)))
    let home = V.first!
    let friend = V.mask(g.next(home))
    let forest = Rooted.forest(g.only(v => v != home), from: List(g.next(home)))
    var route = Map<V, List<V>>()

    fn pair_up(u: V) -> ?List<V>:
        let open = List(
            chain(if friend[u]: [List<V>()] else: [], forest.children(u).filter_map(pair_up))
        )
        for pair in open.chunks(2) if pair.len == 2:
            let path = pair.first! + [u] + pair.last!.rev
            route[path.first!] = path
            route[path.last!] = path.rev
        open.len.is_odd().then(open.last! + [u])

    if forest.roots.any(r => pair_up(r).is_some()):
        return
    while let u = V.at(get_move()):
        for v in route[u].drop(1):
            make_move(Int(v))
        make_move(Int(home))
