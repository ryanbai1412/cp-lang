// Codeforces 1984 E: one shuffle roots the tree at a chosen vertex and reattaches each detached subtree,
// so the leaves that survive form an independent set, plus the new root's side when it was a leaf.
// Rooting at a leaf r, the answer is the maximum independent set of the rest plus r itself, and
// the greedy "take a vertex iff no child is taken" is optimal from every root at once by rerooting.
@!multitest
domain V = 1..=read(2..=2e5)
let g = Graph.undirected(read<[V.count - 1](V, V)>())
let greedy = Rooted(g, V.first).reroot(
    (count: +, taken: or),
    finish: s => (count: s.count + (not s.taken).int, taken: not s.taken),
)
print(V.filter(r => g.deg(r) == 1).max(r => greedy[r].count - greedy[r].taken.int + 1))
