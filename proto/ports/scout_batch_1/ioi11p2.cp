// IOI 2011 Race: in a tree of n cities with road lengths, find a path of
// total length exactly k that uses the fewest roads. Print that count, or -1.
let n: 1..=2e5 = read()
let k: 1..=1e6 = read()
domain City = 0..n
let roads: [n - 1](u: City, v: City, len: 0..=1e6) = read()         // WISH F21: a count that is an expression
let tree = Rooted(roads, City.first, weight: .len)                  // WISH F12, F16: lengths on a rooted tree

// Small-to-large over the subtrees. reach[v] maps a distance from the root
// to the fewest roads from the root, over the cities under v. A path that
// turns at v joins two of those cities, taken from different children.
var reach: [City]HashMap<Int, Int> = []
var fewest: ?Int = none
for v in tree.preorder.rev:                 // children before parents, so no recursion
    var here: HashMap<Int, Int> = [tree.dist(v): tree.depth(v)]             // WISH F21: a map literal
    for child in tree.children(v):
        var there = take(reach[child])                              // WISH F17: move a slot's value out
        if there.len > here.len: swap(here, there)
        for (far, hops) in there:
            if let other = here.get(k + 2 * tree.dist(v) - far):
                fewest min= hops + other - 2 * tree.depth(v)
        for (far, hops) in there: here[far] min= hops               // WISH F18: min= on a key not there yet
    reach[v] = here
print(fewest ?? -1)
