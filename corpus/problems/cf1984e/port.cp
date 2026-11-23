// Source: Codeforces 1984E (Shuffle), https://codeforces.com/contest/1984/problem/E
// Problem: a tree of n <= 2e5 nodes (sum n <= 3e5, multitest) is shuffled once: pick a root V,
//   remove it, shuffle each remaining subtree the same way and hang their roots under V. Print
//   the most leaves (degree-1 nodes, a root with one child included) the new tree can have.
// Solution: root at an original leaf r, which stays a leaf; the shuffle of T - r can then make
//   a maximum independent set of T - r into leaves, so the answer is max over leaves r of
//   1 + MIS(T - r). Bottom-up greedy (take a node iff no child is taken) is an optimal MIS on a
//   tree; `reroot` gives its (count, root taken) for every root in O(n), and MIS(T - r) is that
//   count minus 1 if r was taken.
@!multitest
let n: 2..=200_000 = read()
domain V = 1..=n
let g = Graph.undirected(read<[n - 1](V, V)>())
let t = Rooted(g, V.first!)

let greedy = t.reroot(
    Monoid(
        (count: 0, taken: false),
        (a, b) => (count: a.count + b.count, taken: a.taken or b.taken),
    ),
    (c, _, _) => c,
    (s, _) => (count: s.count + Int(not s.taken), taken: not s.taken),
)
print([greedy[r].count - Int(greedy[r].taken) + 1 for r in V if g[r].len == 1].max()!)
