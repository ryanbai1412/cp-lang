// Source: CCC 2010 S4 (Animal Farm), https://dmoj.ca/problem/ccc10s4
// Problem: M <= 100 pens, each a polygon of 3..=8 numbered corner posts (<= 1000) with a cost
//   (<= 5000) per edge; an edge listed by two pens joins them, one listed once faces outside.
//   One animal per pen; print the least total cost of edges to break so that all of them can
//   meet in one pen or outside.
// Solution: pens and the outside are vertices, edges are weighted edges; the answer is the
//   cheaper of a minimum spanning tree of the pens alone and of the pens plus the outside.
//   A Map matches edges by their sorted post pair, then Kruskal with a Dsu. O(E log E).
let pens: 1..=100 = read()
domain Pen = 0..=pens  // pen 0 is the outside

// a fence is one side of two pens, or of one pen and the outside
var fences: List<(cost: Int, u: Pen, v: Pen)> = []
var open = Map<(Int, Int), (cost: Int, pen: Pen)>()  // sides seen once so far
for pen in Pen.at(1)!..=Pen.last!:
    let m: 3..=8 = read()
    let posts: [0..m]1..=1000 = read()
    let costs: [0..m]1..=5000 = read()
    for j in 0..m:
        let (a, b) = (posts[j], posts[(j + 1) % m])
        let side = (min(a, b), max(a, b))
        if let other = open.remove(side): fences.push((costs[j], pen, other.pen))
        else: open[side] = (cost: costs[j], pen:)
for (_, (cost, pen)) in open: fences.push((cost, pen, Pen.first!))
fences.sort()

fn spanning(outside: Bool) -> ?Int:
    var joined = Dsu<Pen>()
    var cost = 0
    for (c, u, v) in fences if (outside or v != Pen.first) and joined.unite(u, v): cost += c
    if joined.components == (if outside: 1 else: 2): cost else: None
print(min(spanning(false), spanning(true))!)
