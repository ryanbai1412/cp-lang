// Source: IOI 2011 Day 1 (Race), https://dmoj.ca/problem/ioi11p2
// limits: 1400 ms, 256 MB
// Problem: a tree of n <= 2e5 cities whose n - 1 roads have lengths 0..=1e6. Find a path of total
//   length exactly k <= 1e6 that uses the fewest roads; print that count, or -1 if none exists.
// Solution: root the tree at city 0; a path turns at its top city v and joins two cities under
//   different children of v. reach[v] maps a distance from the root to the fewest roads from the
//   root over cities under v; children are merged small-to-large, each entry (far, hops) of the
//   smaller map first looking up k + 2 dist(v) - far in the larger, giving
//   hops + other - 2 depth(v) roads. Children go before parents (`tree.at.rev`), no recursion:
//   O(n log n) hash operations.
// The IOI grader calls best_path(N, K, H, L); like orig.cpp's main, this port reads n and k,
//   then the n - 1 roads as `u v len` lines.
// Bounds: n: 1..=2e5; k: 1..=1e6
let n: 1..=2e5 = read()
let k: 1..=1e6 = read()
domain City = 0..n
domain Road = 1..n

fn random_tree() -> [Road](u: City, v: City, len: 0..=1e6):
    [Road] t => (City(rand(0, t.label - 1)), City(t.label), rand(0, 1e6))

let roads: [Road](u: City, v: City, len: 0..=1e6) = read(gen: random_tree)
let tree = Rooted(Graph.undirected(roads).weighted(a => roads[a.e].len), City.first!)

// Small-to-large over the subtrees. reach[v] maps a distance from the root
// to the fewest roads from the root, over the cities under v. A path that
// turns at v joins two of those cities, taken from different children.
var reach: [City]HashMap<Int, Int> = []
var fewest: ?Int = None
for v in tree.at.rev:  // children before parents, so no recursion
    let (far_v, deep_v) = (tree.dist(tree.root, v), tree.depth(v))
    var here: HashMap<Int, Int> = [far_v: deep_v]
    for child in tree.children(v):
        var there: HashMap<Int, Int> = []
        swap(mut there, mut reach[child])
        if there.len > here.len: swap(mut here, mut there)
        for (far, hops) in there:
            if let other = here.get(k + 2 * far_v - far):
                fewest min= hops + other - 2 * deep_v
        for (far, hops) in there: here[far] min= hops
    swap(mut reach[v], mut here)
print(fewest ?? -1)
