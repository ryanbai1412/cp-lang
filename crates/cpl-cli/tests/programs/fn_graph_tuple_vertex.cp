// A fn graph's vertices are what it takes: over `((R, C)) -> …` they are
// the inline product `(R, C)`, a domain, so `components` can key by them,
// and its result takes `comp[r, c]` as well as `comp[(r, c)]` (decision 159).
use std.graph.{components}
domain R = 0..2
domain C = 0..3
let wall: [R, C]Bool = [R, C] (r, c) => Int(c) == 1
fn nb(p: (R, C)) -> List<(R, C)> = p.adj4.filter(q => wall[q] == wall[p]).collect()
let (domain K, comp) = components(nb)
print(K.count, yes(comp[(R(0), C(0))] == comp[R(1), C(0)]))
let (domain J, part) = components((p: (R, C)) => p.adj4.filter(q => wall[q] == wall[p]))
print(J.count, yes(part[R(0), C(0)] != part[R(1), C(2)]))
