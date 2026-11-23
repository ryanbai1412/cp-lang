// IOI 2011 Race: in a tree of n cities with road lengths, find a path of
// total length exactly k that uses the fewest roads. Print that count, or -1.
let n: 1..=2e5 = read()
let k: 1..=1e6 = read()
domain City = 0..n
domain Road = 1..n

fn random_tree() -> [Road](u: City, v: City, len: 0..=1e6):
    [Road] t => (City(rand(0, t.label - 1)), City(t.label), rand(0, 1e6))

let roads: [Road](u: City, v: City, len: 0..=1e6) = read(gen: random_tree)
let tree = Rooted.weighted(Graph.undirected(roads), City.first, a => roads[a.e].len)

// Small-to-large over the subtrees. reach[v] maps a distance from the root
// to the fewest roads from the root, over the cities under v. A path that
// turns at v joins two of those cities, taken from different children.
var reach: [City]HashMap<Int, Int> = []
var fewest: ?Int = none
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
