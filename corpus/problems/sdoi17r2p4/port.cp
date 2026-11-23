// Source: SDOI 2017 Round 2 Genius Hacker (Luogu P3783), https://dmoj.ca/problem/sdoi17r2p4
// Problem: T <= 10 tests: a directed graph, n, m <= 5e4 (self-loops and multi-edges allowed), each
//   edge with a cost c <= 2e4 and a word, a node of a trie of k <= 2e4 nodes. Taking an edge costs
//   c plus the LCP length of its word and the previous edge's word (the empty word at the start,
//   town 1). Print the cheapest cost from town 1 to every other town.
// Solution: Dijkstra over edges, without an arc per pair of edges at a town: sort the edges at
//   each town by DFS order of their words, so the LCP of two is the min LCP of neighbours between
//   them. Four prefix/suffix chains along the sorted list (arrived-by, may-leave-by, each way)
//   step between neighbours at cost 0, and cross from arrived to leave at their LCP (an LCA
//   depth). O(m) nodes and arcs, O(m log m).
// original: corpus/submissions/5515890.cpp
let tests: 1..=3 = read()
for _ in 0..tests:
    let n: 2..=5e4 = read()
    let m: n - 1..=5e4 = read()
    let k: 1..=2e4 = read()
    domain Town = 1..=n
    domain Road = 0..m
    domain Word = 1..=k
    domain Letter = 1..k

    fn reaching() -> [Road](u: Town, v: Town, cost: 1..=5e4, word: Word):  // generator: every town can be reached
        [Road] r => (
            if r.idx < n - 1: (
                Town(rand(1, r.idx + 1)),
                Town(r.idx + 2),
                rand(1, 50000),
                Word(rand(1, k)),
            ) else: (Town(rand(1, n)), Town(rand(1, n)), rand(1, 50000), Word(rand(1, k)))
        )
    fn random_tree() -> [Letter](u: Word, v: Word, ch: 1..=2e4):
        [Letter] l => (Word(rand(1, l.label)), Word(l.label + 1), l.label)

    let roads: [Road](u: Town, v: Town, cost: 1..=5e4, word: Word) = read(gen: reaching)
    let letters: [Letter](u: Word, v: Word, ch: 1..=2e4) = read(gen: random_tree)
    let words = Rooted(Graph.undirected(letters), Word.first!)

    // The roads touching each town, in dictionary order of their words.
    // Then the common prefix of any two is the shortest one shared by
    // neighbours anywhere between them.
    var ends: [Town]List<(at: Int, road: Road, leaving: Bool)> = []
    for r in Road:
        ends[roads[r].u].push((words.pos(roads[r].word).ord, r, true))
        ends[roads[r].v].push((words.pos(roads[r].word).ord, r, false))

    // A shortest-path graph over the roads. To avoid an arc for every pair
    // of roads at a town, four chains run along the town's sorted roads:
    //   0: arrived by this road or an earlier one    2: may leave by this road or a later one
    //   1: arrived by this road or a later one       3: may leave by this road or an earlier one
    // and between neighbours, chain 0 steps over to chain 2 and chain 1 to
    // chain 3 at the price of their common prefix.
    domain Node = 0..9 * m + 1
    fn taken(r: Road) -> Node: Node(r.idx)
    fn link(slot: Int, chain: 0..=3) -> Node: Node(m + 4 * slot + chain)
    let start = Node(9 * m)
    var arcs: List<(u: Node, v: Node, w: Int)> = []
    var slot = 0
    for v in Town:
        let here = ends[v].sorted()
        for i in 0..here.len:
            let (_, r, leaving) = here[i]
            let s = slot + i
            if leaving:
                arcs.push((link(s, 2), taken(r), roads[r].cost))
                arcs.push((link(s, 3), taken(r), roads[r].cost))
                if v == Town.first: arcs.push((start, taken(r), roads[r].cost))
            else:
                arcs.push((taken(r), link(s, 0), 0))
                arcs.push((taken(r), link(s, 1), 0))
            if i + 1 < here.len:
                let shared = words.depth(words.lca(roads[r].word, roads[here[i + 1].road].word))
                arcs.push((link(s, 0), link(s + 1, 0), 0))
                arcs.push((link(s + 1, 1), link(s, 1), 0))
                arcs.push((link(s, 2), link(s + 1, 2), 0))
                arcs.push((link(s + 1, 3), link(s, 3), 0))
                arcs.push((link(s, 0), link(s + 1, 2), shared))
                arcs.push((link(s + 1, 1), link(s, 3), shared))
        slot += here.len

    let es: [0..arcs.len](u: Node, v: Node, w: Int) = [0..arcs.len] i => arcs[i]
    let g = Graph.directed(es)
    let dist = dijkstra(g, from: [(start, 0)])
    var best: [Town]?Int = None
    for r in Road:
        if let d = dist[taken(r)]: best[roads[r].v] min= d
    for v in Town:
        if v != Town.first: print(best[v]!)
