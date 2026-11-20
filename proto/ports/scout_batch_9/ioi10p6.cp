// original: proto/ports/scout_batch_9/harness/ioi10p6.cpp
// IOI 2010 Traffic Congestion: cities joined into a tree by roads, with
// fans living in each. Everyone drives to the arena city. Pick the city
// that makes the busiest road into it carry as few fans as possible.
let n: 1..=1e6 = read()
domain City = 0..n
domain Road = 1..n

fn random_tree() -> [Road](u: City, v: City):
    [Road] r => (City(rand(0, r.label - 1)), City(r.label))

let fans: [City]1..=2000 = read()
let roads: [Road](u: City, v: City) = read(gen: random_tree)
let tree = Rooted(roads, City.first)

// below[c]: the fans in c's subtree; they all cross the road above c
var below: [City]Int = [City] c => fans[c]
for c in tree.preorder.rev:
    if let p = tree.parent(c): below[p] += below[c]

// With the arena at c, the roads into c carry each child's subtree, and
// everyone outside c's subtree.
fn busiest(c: City) -> Int:
    var worst = below[City.first] - below[c]
    for k in tree.children(c): worst max= below[k]
    worst

print(min((busiest(c), c) for c in City).1)
