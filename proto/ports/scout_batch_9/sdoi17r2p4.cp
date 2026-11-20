// original: corpus/submissions/5515890.cpp
// SDOI 2017 Genius Hacker: one-way roads between towns. Every road has a
// cost and a word from a dictionary tree (a word is a vertex, its prefixes
// are its ancestors). Driving one road straight after another also costs
// the length of the longest common prefix of their two words. Print the
// cheapest trip from town 1 to every other town.
let tests: 1..=3 = read()
for _ in 0..tests:
    let n: 2..=5e4 = read()
    let m: n - 1..=5e4 = read()
    let k: 1..=2e4 = read()
    domain Town = 1..=n
    domain Road = 0..m
    domain Word = 1..=k
    domain Letter = 1..k

    fn reaching() -> [Road](u: Town, v: Town, cost: 1..=5e4, word: Word):     // generator: every town can be reached
        [Road] r => (if r.idx < n - 1: (Town(rand(1, r.idx + 1)), Town(r.idx + 2), rand(1, 50000), Word(rand(1, k)))
                     else: (Town(rand(1, n)), Town(rand(1, n)), rand(1, 50000), Word(rand(1, k))))
    fn random_tree() -> [Letter](u: Word, v: Word, ch: 1..=2e4):
        [Letter] l => (Word(rand(1, l.label)), Word(l.label + 1), l.label)

    let roads: [Road](u: Town, v: Town, cost: 1..=5e4, word: Word) = read(gen: reaching)
    let letters: [Letter](u: Word, v: Word, ch: 1..=2e4) = read(gen: random_tree)
    let words = Rooted(letters, Word.first)

    // The roads touching each town, in dictionary order of their words.
    // Then the common prefix of any two is the shortest one shared by
    // neighbours anywhere between them.
    var ends: [Town]List<(at: Int, road: Road, leaving: Bool)> = []
    for r in Road:
        ends[roads[r].u].push((words.pos(roads[r].word), r, true))
        ends[roads[r].v].push((words.pos(roads[r].word), r, false))

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
    var arcs: List<(u: Node, v: Node, cost: Int)> = []
    var slot = 0
    for v in Town:
        let here = sorted(ends[v])
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

    let dist = dijkstra(Graph.directed(arcs), start)
    var best: [Town]?Int = none
    for r in Road:
        if let d = dist[taken(r)]: best[roads[r].v] min= d
    for v in Town:
        if v != Town.first: print(best[v]!)
