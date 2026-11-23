// Source: Codeforces 2147H (Maxflow GCD Coloring), https://codeforces.com/contest/2147/problem/H
// Problem: an undirected graph, n <= 50, positive capacities <= 1e6 (sum of n^4 <= 50^4;
//   multitest). Color the vertices with the fewest colors so that each color's induced subgraph
//   is good: some d >= 2 divides maxflow(u, v) for all its pairs. Print the coloring.
// Solution: the max flows are exactly the Gomory-Hu tree's weights, so the graph is good iff
//   their gcd g is not 1 (g = 0 with no edges): one color. Else two suffice (Gallai): split V so
//   every vertex has an even number of odd-weight neighbours on its side; then every cut inside
//   a side is even. The split peels an odd-degree vertex after complementing the odd edges among
//   its neighbours, and puts it back on the side where it is even. n - 1 flows plus O(n^2) bitset
//   steps.
use std.flow.gomory_hu

@!multitest
let (n, m): (1..=50, 0..=1225) = read()
domain V = 1..=n
let edges = read<[m](u: V, v: V, w: 1..=1_000_000)>()

var g = 0
for e in gomory_hu(edges): g = gcd(g, e.w)

let colors: List<Bitset<V>> = if g != 1:
    [V.all]
else:
    var odd = adjacency(edges.filter(e => e.w % 2 == 1), into: Bitset)

    fn complement_within(s: Bitset<V>):
        for u in s: odd[u] ^= s.without(u)

    // Gallai's induction: remove an odd-degree v after complementing its neighborhood;
    // on the way back v joins the side where its degree is even.
    fn even_split(alive: Bitset<V>) -> Bitset<V>:
        let v = V.find(
            v => v in alive and (odd[v] & alive).len % 2 == 1
        ) ?? return Bitset<V>.empty()
        let rest = alive.without(v)
        let nbr = odd[v] & rest
        complement_within(nbr)
        var side = even_split(rest)
        complement_within(nbr)
        if (nbr & side).len % 2 == 0: side.add(v)
        side

    let side = even_split(V.all)
    [V.all - side, side]

print(colors.len)
for c in colors:
    print(c.len)
    print(c)
