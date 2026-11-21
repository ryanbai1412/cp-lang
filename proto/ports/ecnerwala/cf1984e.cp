@!multitest
let n: 2..=200_000 = read()
domain V = 1..=n
let g = Graph.undirected(read<[n - 1](V, V)>())
let t = Rooted(g, V.first)

let greedy = t.reroot(
    Monoid(
        (count: 0, taken: false),
        (a, b) => (count: a.count + b.count, taken: a.taken or b.taken),
    ),
    (c, _, _) => c,
    (s, _) => (count: s.count + Int(not s.taken), taken: not s.taken),
)
print([greedy[r].count - Int(greedy[r].taken) + 1 for r in V if g[r].len == 1].max())
