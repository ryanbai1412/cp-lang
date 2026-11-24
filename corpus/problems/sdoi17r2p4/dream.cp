// SDOI 2017 "Dictionary Trie Road Network": entering road b after road a costs
// the depth of the LCA of their words in a trie; shortest paths from 1.
// Source: SDOI 2017 Round 2 Genius Hacker (Luogu P3783), https://dmoj.ca/problem/sdoi17r2p4
// Dijkstra over roads, but a pair of roads at a town would be O(deg^2) arcs. Sort a town's road
// ends by the preorder of their words: the LCP of two is the least LCP of neighbours between
// them, so an arrival chain and a departure chain per direction carry every turn in O(deg).
@!multitest
let (n, m, k): (2..=5e4, 1..=5e4, 1..=2e4) = read()
domain V = 1..=n
domain Word = 1..=k
domain Road = [m]
let roads: [Road](u: V, v: V, w: 0..=2e4, word: Word) = read()
let trie = Rooted(
    Graph.undirected(read<[k - 1](u: Word, v: Word, c: 1..=2e4)>()),
    root: Word.first!,
)
let leave = Graph.directed(roads)
let enter = leave.transpose()
fn lcp(a: Road, b: Road) -> Int = trie.depth(trie.lca(roads[a].word, roads[b].word))

// node r.idx: having just driven road r; then two chains per town and direction
domain Node = [9 * m]
fn driven(r: Road) -> Node = Node(r.idx)
var arcs = List<(from: Node, arc: (to: Node, w: Int))>()
var used = m
for x in V:
    let ends = List(
        chain(
            enter.out(x).map(a => (road: a.e, arrives: true)),
            leave.out(x).map(a => (road: a.e, arrives: false)),
        )
    ).sorted(by: it => trie.pos(roads[it.road].word))
    for line in [ends, List(ends.rev)]:
        let t = line.len
        let (arrived, departing) = ((i: Int) => Node(used + i), (i: Int) => Node(used + t + i))
        for (i, it) in line.items:
            if it.arrives: arcs.push((from: driven(it.road), arc: (to: arrived(i), w: 0)))
            else: arcs.push((from: departing(i), arc: (to: driven(it.road), w: roads[it.road].w)))
            if i + 1 < t:
                arcs.push((from: arrived(i), arc: (to: arrived(i + 1), w: 0)))
                arcs.push((from: departing(i), arc: (to: departing(i + 1), w: 0)))
                arcs.push(
                    (
                        from: arrived(i),
                        arc: (to: departing(i + 1), w: lcp(it.road, line[i + 1].road)),
                    )
                )
        used += 2 * t

let dist = dijkstra(
    Graph.of(arcs),
    from: leave.out(V.first!).map(a => (driven(a.e), roads[a.e].w)),
).dist
for v in V.drop(1):
    print(enter.out(v).filter_map(a => dist[driven(a.e)]).min()!)
