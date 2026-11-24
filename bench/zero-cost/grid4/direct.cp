// The same DP on one flat list, indexed by hand: every other cost (the
// domain loops, `next!`, the prefix sums) is the abstract one's.
let n: 1..=50 = read()
let m: 1..=50 = read()
domain Row = 1..=n
domain Col = 1..=m
let raisins: [Row, Col]1..=1000 = read()
let pre = prefix(raisins)

var pay: List<0..=4e9> = [0 for _ in 0..n * n * m * m]
fn at(r1: Row, r2: Row, c1: Col, c2: Col) -> Int = ((r1.ord * n + r2.ord) * m + c1.ord) * m + c2.ord
unchecked:
    for r1 in Row.rev:
        for r2 in r1..=Row.last!:
            for c1 in Col.rev:
                for c2 in c1..=Col.last!:
                    if r1 == r2 and c1 == c2: continue
                    var best = Int.MAX
                    for r in r1..r2: best min= pay[at(r1, r, c1, c2)] + pay[at(r.next!, r2, c1, c2)]
                    for c in c1..c2: best min= pay[at(r1, r2, c1, c)] + pay[at(r1, r2, c.next!, c2)]
                    pay[at(r1, r2, c1, c2)] = best + pre.fold(r1..=r2, c1..=c2)
print(pay[at(Row.first!, Row.last!, Col.first!, Col.last!)])
