// Source: IOI 2010 Day 2 (Traffic Congestion), https://dmoj.ca/problem/ioi10p6
// Problem: a tree of n <= 1e6 cities, P[i] >= 1 fans in city i (2e9 in total). Everyone
//   travels to the arena city; print a city whose most congested road (fans crossing it)
//   carries as few fans as possible.
// Solution: root at city 0 and sum the fans of each subtree. With the arena at c, the road to
//   each child k carries below[k] and the road to the parent the total - below[c]; take the
//   city with the smallest maximum. O(n).
// original: proto/ports/scout_batch_9/harness/ioi10p6.cpp
let n: 1..=1e6 = read()
domain City = 0..n
domain Road = 1..n

fn random_tree() -> [Road](u: City, v: City):
    [Road] r => (City(rand(0, r.label - 1)), City(r.label))

let fans: [City]1..=2e9 = read()
assert fans.sum() <= 2e9
let roads: [Road](u: City, v: City) = read(gen: random_tree)
let tree = Rooted(Graph.undirected(roads), City.first!)

// below[c]: the fans in c's subtree; they all cross the road above c
var below: [City]Int = [City] c => fans[c]
for c in tree.at.rev:
    if let p = tree.parent(c): below[p] += below[c]

// With the arena at c, the roads into c carry each child's subtree, and
// everyone outside c's subtree.
fn busiest(c: City) -> Int:
    var worst = below[City.first!] - below[c]
    for k in tree.children(c): worst max= below[k]
    worst

print([(busiest(c), c) for c in City].min()!.1)
