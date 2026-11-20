// cp: 0.1 1f59ac0+dirty
// CCC 2010 S4 Animal Farm: pens are polygons whose corners are numbered
// posts; a fence between two posts has a cost to remove. Remove the cheapest
// fences so that all animals can meet, either inside or by all getting
// outside: a minimum spanning tree over the pens, with or without the
// outside as an extra vertex.
use std.dsu.{Dsu}

let pens: 1..=100 = read()
domain Pen = 0..=pens                   // pen 0 is the outside
domain Post = 1..=1000
type Side = (a: Post, b: Post, pen: Pen, cost: Int)

var sides: List<Side> = []
for pen in Pen.first.next!..:
    let m: 3..=8 = read()
    let posts: [m]Post = read() where all(posts[j] != posts[(j + 1) % m] for j in 0..m)
    let costs: [m]1..=5000 = read()
    for j in 0..m:
        let (a, b) = (posts[j], posts[(j + 1) % m])
        sides.push((min(a, b), max(a, b), pen, costs[j]))

type Fence = (cost: Int, u: Pen, v: Pen)
var fences: List<Fence> = []
for wall in runs(sorted(sides, by: s => (s.a, s.b)), same: (s, t) => (s.a, s.b) == (t.a, t.b)):
    let cost = wall[wall.len - 1].cost
    if wall.len == 2: fences.push((cost, wall[0].pen, wall[1].pen))
    if wall.len == 1: fences.push((cost, wall[0].pen, Pen.first))
fences.sort()

fn spanning(outside: Bool) -> Int:
    var joined = Dsu<Pen>()
    sum(f.cost for f in fences if (outside or f.v != Pen.first) and joined.unite(f.u, f.v))
print(min(spanning(false), spanning(true)))
