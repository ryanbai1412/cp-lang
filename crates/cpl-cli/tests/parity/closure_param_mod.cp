// A closure param is its own variable: `a * b % m` in a Monoid closure
// must not borrow a 32-bit range from the maker's local in the same slot.
let (n, m): (1..=1e5, 2..=1e9) = read()
domain V = 1..=n
let tree = Rooted(Graph.undirected(read<[n - 1](V, V)>()), V.first!)

let ways = tree.reroot(Monoid(1, (a, b) => a * b % m), (x, _, _) => (x + 1) % m, (u, _) => u)
for v in V: print(ways[v])
