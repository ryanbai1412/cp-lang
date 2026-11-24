// Source: Baltic OI 2018 Day 2 (Paths), https://dmoj.ca/problem/btoi18p6
// Problem: an undirected graph, n, m <= 3e5, each vertex coloured 1..k with k <= 5. Count the
//   paths of at least two vertices whose colours are all distinct, each direction separately.
// Solution: distinct colours make a path simple and at most k long. ways[s, v] counts paths
//   ending at v that use exactly the colour set s: 1 if s is v's colour alone, else the sum
//   of ways[s minus v's colour, u] over neighbours u. Subsets in increasing order come after
//   their own subsets; add every ways[s, v] with |s| >= 2. O(2^k (n + m)).
// Bounds: n: 1..=3e5; m: 0..=3e5; k: 1..=5; colour: 1..=k; edges.u, edges.v: 1..=n
let n: 1..=3e5 = read()
let m: 0..=3e5 = read()
let k: 1..=5 = read()
domain V = 1..=n
domain Colour = 1..=k
type Used = Subset<Colour>
let colour: [V]Colour = read()
let edges: [m](u: V, v: V) = read()

var adj: [V]List<V> = []
for (u, v) in edges:
    adj[u].push(v)
    adj[v].push(u)

// ways[s, v]: paths ending at v whose colours are exactly the set s.
// A subset comes after all its own subsets, so one pass in order fills it.
var ways: [Used, V]Int = 0
for s in Used:
    for v in V:
        if colour[v] in s:
            let rest = s.without(colour[v])
            ways[s, v] = if rest == Used.empty: 1 else: sum(ways[rest, u] for u in adj[v])
print(sum(ways[s, v] for s in Used for v in V if s.len >= 2))
