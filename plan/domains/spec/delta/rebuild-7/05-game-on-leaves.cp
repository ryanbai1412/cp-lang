// Codeforces 1363C Game On Leaves
// in: t; per test n and special node x, then n-1 tree edges "u v". out: per test the winner, "Ayush" or "Ashish".
@!multitest
domain V = 1..=read<1..=1000>()
let x = read<V>()
let g = Graph.undirected(read<[V.count - 1](u: V, v: V)>())
print(yes(g.deg(x) <= 1 or V.count % 2 == 0, answers: ("Ayush", "Ashish")))
