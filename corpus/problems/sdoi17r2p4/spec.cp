// SDOI 2017 "Dictionary Trie Road Network": entering road b after road a costs
// w(b) plus the depth of the LCA of their words in a trie; shortest paths from 1
// over roads. At a vertex, sort the roads entering and leaving it by their
// words' trie preorder: the LCA depth of two words is the minimum over the
// adjacent pairs between them, so two prefix chains and a cross arc per gap
// (one pass each way) carry every turn in O(deg) arcs instead of O(deg^2).
@!multitest
let (n, m, k) = read<(2..=5e4, 1..=5e4, 1..=2e4)>()
domain V = 1..=n
domain Word = 1..=k
domain Road = [m]
let roads = read<[Road](u: V, v: V, w: 0..=2e4, word: Word)>()
let trie = Rooted(Graph.undirected(read<[k - 1](u: Word, v: Word, c: 1..=2e4)>()), root: Word.first!)
let lca = LcaTable(trie)
let leave = Graph.directed(roads)
let enter = leave.transpose()

// Each road, then two chain nodes per road end per pass. Ideally
// `domain Node = Road + Aux`, but sum domains don't construct yet.
domain Node = [9 * m]
let at = (r: Road) => Node(r.idx)
var arcs = List<(from: Node, arc: (to: Node, w: Int))>()
var used = 0
for x in V:
    let here = List(
        chain(
            enter.out(x).map(a => (road: a.e, enters: true)),
            leave.out(x).map(a => (road: a.e, enters: false)),
        )
    ).sorted(by: it => trie.pos(roads[it.road].word))
    for line in [here, List(here.rev)]:
        let t = line.len
        let (pre, suf) = ((i: Int) => Node(m + used + i), (i: Int) => Node(m + used + t + i))
        for (i, it) in line.items:
            if it.enters:
                arcs.push((from: at(it.road), arc: (to: pre(i), w: 0)))
            else:
                arcs.push((from: suf(i), arc: (to: at(it.road), w: roads[it.road].w)))
            if i + 1 < t:
                let gap = trie.depth(lca.lca(roads[it.road].word, roads[line[i + 1].road].word))
                arcs.push((from: pre(i), arc: (to: pre(i + 1), w: 0)))
                arcs.push((from: suf(i), arc: (to: suf(i + 1), w: 0)))
                arcs.push((from: pre(i), arc: (to: suf(i + 1), w: gap)))
        used += 2 * t

let dist = dijkstra(
    Graph.of(arcs),
    from: leave.out(V.first!).map(a => (at(a.e), roads[a.e].w)),
).dist
for v in V.drop(1):
    print(enter.out(v).filter_map(a => dist[at(a.e)]).min()!)
