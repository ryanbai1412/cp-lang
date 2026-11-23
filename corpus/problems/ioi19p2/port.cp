// original: proto/ports/scout_batch_9/harness/ioi19p2.cpp
// checker: proto/ports/scout_batch_9/harness/ioi19p2_check.cpp
// IOI 2019 Split the Attractions: a connected map of attractions. Split
// them into three parts of given sizes (a, b and the rest) so that at
// least two of the parts are connected. Print each attraction's part, or
// all zeros.
let n: 3..=1e5 = read()
let m: n - 1..=2e5 = read()
let a: 1..=n - 2 = read()
let b: 1..=n - 2 = read()
assert a + b < n  // and the third part is the rest
domain Spot = 0..n
domain Link = 0..m

// generator: a bushy tree, then extra links that mostly lead to spot 0, so
// that some maps cannot be split
fn bushy() -> [Link](u: Spot, v: Spot):
    [Link] l => (
        if l.idx < n - 1: (
            Spot(rand(0, l.idx) / 3),
            Spot(l.idx + 1),
        ) elif rand(0, 3) > 0: (Spot.first!, Spot(rand(0, n - 1))) else: (
            Spot(rand(0, n - 1)),
            Spot(rand(0, n - 1)),
        )
    )

let links: [Link](u: Spot, v: Spot) = read(gen: bushy)
let land = Graph.undirected(links)

// Make the two smaller parts connected; the largest takes what is left.
let by_size = [(a, 1), (b, 2), (n - a - b, 3)].sorted()
let (small, small_part) = by_size[0]
let (mid, mid_part) = by_size[1]
let (_, big_part) = by_size[2]

// Take any spanning tree and its hub: the spot whose removal leaves no
// branch with more than half of the spots.
let tree = Rooted(land, Spot.first!)
var hub = Spot.first!
for v in Spot:
    if 2 * (n - tree.size(v)) <= n and all(2 * tree.size(c) <= n for c in tree.children(v)): hub = v

// Group the branches around the hub, and merge groups along any link for
// as long as both are too small to hold the small part. A group that gets
// big enough holds it, and everything else still hangs together through
// the hub and has room for the middle part.
var group = Dsu<Spot>()
for v in Spot:
    if let p = tree.parent(v) and v != hub and p != hub: group.unite(v, p)
for (u, v) in links:
    if u != hub and v != hub and group.size(u) < small and group.size(v) < small: group.unite(u, v)
var seed: ?Spot = None
for v in Spot:
    if v != hub and group.size(v) >= small: seed = v

// Mark `count` spots as `part`: a connected patch grown outwards from
// `from`, staying on from's side of the split.
var part: [Spot]Int = 0
fn claim(from: Spot, side: Spot, count: Int, mark: Int):
    var patch = [from]
    part[from] = mark
    var i = 0
    while i < patch.len:
        for (w, _) in land[patch[i]]:
            if patch.len < count and part[w] == 0 and group.same(w, side) == group.same(from, side):
                part[w] = mark
                patch.push(w)
        i += 1

if let s = seed:
    claim(s, s, small, small_part)
    claim(hub, s, mid, mid_part)
    for v in Spot:
        if part[v] == 0: part[v] = big_part
print(part)
