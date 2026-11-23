// CF 1324F Maximum White Subtree: rerooted tree DP. A vertex keeps a
// neighbour's side only when it helps: best[v] = own[v] + sum max(0, side).
let n: 2..=2e5 = read()
domain V = 1..=n
let color = read<[V]0..=1>()
let g = Graph.undirected(read<[n - 1](u: V, v: V)>())
let own = [V] v => if color[v] == 1: 1 else: -1

let t = Rooted(g, root: V.first!)
let best = t.reroot(+, lift: (x, _, _) => max(x, 0), finish: (s, v) => s + own[v])
print(best)
