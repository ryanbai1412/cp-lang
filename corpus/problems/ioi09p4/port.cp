// IOI 2009 Raisins: a chocolate slab of n by m pieces, each with some
// raisins. Cut it into single pieces with straight full cuts; a cut costs
// the raisins on the slab being cut. Print the least total cost.
// pay(r1, r2, c1, c2) = cheapest way to take rows r1..=r2, columns c1..=c2
// apart: its raisins, plus the best first cut.
// original: corpus/submissions/4817803.cpp
let n: 1..=50 = read()
let m: 1..=50 = read()
domain Row = 1..=n
domain Col = 1..=m
let raisins: [Row, Col]1..=1000 = read()
let pre = prefix(raisins)

@memo
fn pay(r1: Row, r2: Row, c1: Col, c2: Col) -> Int:
    if r1 == r2 and c1 == c2: return 0
    var best: ?Int = none
    for r in r1..r2: best min= pay(r1, r, c1, c2) + pay(r.next!, r2, c1, c2)
    for c in c1..c2: best min= pay(r1, r2, c1, c) + pay(r1, r2, c.next!, c2)
    best! + pre.sum(r1..=r2, c1..=c2)
print(pay(Row.first, Row.last, Col.first, Col.last))
