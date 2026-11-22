// EGOI 2022 "Social Engineering": Maria (vertex 1) and an adversary walk a
// graph without reusing edges; whoever cannot move loses. Win if possible.
@grader fn get_move() -> Int
@grader fn make_move(v: Int)

pub fn social_engineering(n: Int, edges: List<(Int, Int)>):
    domain V = 1..=n
    let g = Graph.undirected(edges.map(e => (u: V(e.0), v: V(e.1))))
    let home = V.first!
    let friend = V.mask(g.next(home))
    let forest = dfs(g.only(v => v != home), from: g.next(home)).forest()
    var route = Map<V, List<V>>()

    fn pair_up(u: V) -> ?List<V>:
        let open = chain(if friend[u]: [List<V>()] else: [], forest.children(u).filter_map(pair_up)).collect()
        for (x, y) in open.chunks_exact(2):
            let path = x + [u] + y.rev
            route[path.first!] = path
            route[path.last!] = path.rev
        open.len.is_odd().then(open.last! + [u])

    if forest.roots.any(r => pair_up(r).is_some()):
        return
    while let u = V.at(get_move()):
        for v in route[u].skip(1):
            make_move(Int(v))
        make_move(Int(home))
