// CCC 2010 S4 Animal Farm: pens are polygons whose corners are numbered
// posts; a fence between two posts has a cost to remove. Remove the cheapest
// fences so that all animals can meet, either inside or by all getting
// outside: a minimum spanning tree over the pens, with or without the
// outside as an extra vertex.
let pens: 1..=100 = read()
domain Pen = 0..=pens                   // pen 0 is the outside

var sides: List<(a: Int, b: Int, cost: Int, pen: Pen)> = []
for pen in Pen if pen != Pen.first:
    let m: 3..=8 = read()
    let posts: [0..m]1..=1000 = read()
    let costs: [0..m]1..=5000 = read()
    for j in 0..m:
        let (a, b) = (posts[j], posts[(j + 1) % m])
        sides.push((min(a, b), max(a, b), costs[j], pen))
sort(mut sides)

// a fence is one side of two pens, or of one pen and the outside
var fences: List<(cost: Int, u: Pen, v: Pen)> = []
for j in 0..sides.len:
    let (a, b, cost, pen) = sides[j]
    if j + 1 < sides.len and (sides[j + 1].a, sides[j + 1].b) == (a, b): fences.push((cost, pen, sides[j + 1].pen))
    elif j == 0 or (sides[j - 1].a, sides[j - 1].b) != (a, b): fences.push((cost, pen, Pen.first))
sort(mut fences)

fn spanning(outside: Bool) -> ?Int:
    var joined = Dsu<Pen>()
    var cost = 0
    for (c, u, v) in fences if (outside or v != Pen.first) and joined.unite(u, v): cost += c
    if joined.components == (if outside: 1 else: 2): cost else: none
print(min(spanning(false), spanning(true))!)
