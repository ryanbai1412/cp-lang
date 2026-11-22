// SDOI 2017 "Dictionary Trie Road Network": entering road b after road a costs
// the depth of the LCA of their words in a trie; shortest paths from 1.
@!multitest
let (n, m, k) = read<(1..=5e4, 1..=1e5, 1..=2e4)>()
domain V = 1..=n
domain Word = 1..=k
let roads = read<[m](u: V, v: V, w: 1..=2e4, word: Word)>()
let trie = Rooted(Graph.undirected(read<[k - 1](u: Word, v: Word, _: Int)>()), root: Word.first!)
let lca = LcaTable(trie)
let leave = Graph.directed(roads)

fn turn(a: roads.keys) =
    leave.out(roads[a].v).map(b => (to: b.e, w: roads[a].w + trie.depth(lca(roads[a].word, roads[b.e].word))))

let dist = dijkstra(turn, from: leave.out(V.first!).map(b => (b.e, 0))).dist
let best = V.min_of(roads.keys, key: e => roads[e].v, of: e => dist[e].map(d => d + roads[e].w))
print(V.rest.map(v => best[v]!), sep: "\n")
