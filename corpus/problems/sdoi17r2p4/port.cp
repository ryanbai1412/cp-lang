// Source: SDOI 2017 Round 2 Genius Hacker (Luogu P3783), https://dmoj.ca/problem/sdoi17r2p4
// limits: 3000 ms, 500 MB
// Problem: T <= 10 tests: a directed graph, n, m <= 5e4 (self-loops and multi-edges allowed), each
//   edge with a cost 0 <= c <= 2e4 and a word, a node of a trie of k <= 2e4 nodes. Taking an edge costs
//   c plus the LCP length of its word and the previous edge's word (the empty word at the start,
//   town 1). Print the cheapest cost from town 1 to every other town.
// Solution: Dijkstra over roads, without an arc per pair of roads at a town: sort the roads
//   at each town by trie preorder of their words, so the LCP of two is the min LCP of
//   neighbours between them. In each direction along the sorted list an arrived chain and a
//   leave chain step between neighbours at cost 0, and the arrived chain crosses to the next
//   leave node at their LCP (an LCA depth). O(m) nodes and arcs, O(m log m).
// original: corpus/submissions/5515890.cpp
// Bounds: n: 2..=5e4; m: 1..=5e4; k: 1..=2e4; roads.u, roads.v: 1..=n; roads.w: 0..=2e4; roads.word: 1..=k
@!multitest
let (n, m, k): (2..=5e4, 1..=5e4, 1..=2e4) = read()
domain V = 1..=n
domain Word = 1..=k
domain Road = [m]
let roads: [Road](u: V, v: V, w: 0..=2e4, word: Word) = read()
let trie = Rooted(Graph.undirected(read<[k - 1](u: Word, v: Word, c: 1..=2e4)>()), root: Word.first!)
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
                let gap = trie.depth(trie.lca(roads[it.road].word, roads[line[i + 1].road].word))
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
