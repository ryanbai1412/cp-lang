// Source: CCC 2010 S4 (Animal Farm), https://dmoj.ca/problem/ccc10s4
// Problem: M <= 100 pens, each a polygon of 3..=8 numbered corner posts (<= 1000) with a cost
//   (<= 5000) per edge; an edge listed by two pens joins them, one listed once faces outside.
//   One animal per pen; print the least total cost of edges to break so that all of them can
//   meet in one pen or outside.
// Solution: pens and the outside are vertices, fences are weighted edges; the answer is the
//   cheaper of a minimum spanning tree of the pens alone and of the pens plus the outside
//   (`kruskal`), counting only spanning ones. Sides pair up by their sorted posts. O(E log E).
// original: corpus/submissions/5341292.cpp
let pens: 1..=100 = read()
domain Region = 0..=pens
let outside = Region.first!

// a side seen once is open, waiting for the pen on its other side; what stays open faces outside
var fences: List<(u: Region, v: Region, w: Int)> = []
var open = Map<(Int, Int), (pen: Region, cost: Int)>()
for pen in Region.after(outside):
    let m: 3..=8 = read()
    let posts: [m]1..=1000 = read()
    let costs: [m]1..=5000 = read()
    for ((a, b), cost) in zip(posts.cyclic_pairs(), costs):
        let side = (min(a, b), max(a, b))
        if let other = open.remove(side): fences.push((u: other.pen, v: pen, w: cost))
        else: open[side] = (pen:, cost:)
fences += [(u: o.pen, v: outside, w: o.cost) for (_, o) in open]

fn spanning(regions: Int, keep: (u: Region, v: Region, w: Int) -> Bool) -> ?Int:
    let (total, take) = kruskal(fences.filter(keep).collect())
    if take.count(t => t) == regions - 1: total else: None
print(min(spanning(pens, f => f.v != outside), spanning(pens + 1, _ => true))!)
