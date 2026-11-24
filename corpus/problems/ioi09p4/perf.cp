// IOI 2009 P4 (Raisins), tuned: port.cp's recurrence filled bottom-up into one 4-byte table,
// blocks before the blocks that contain them, instead of `@memo` with a seen flag per lookup.
let n: 1..=50 = read()
let m: 1..=50 = read()
domain Row = 1..=n
domain Col = 1..=m
let raisins: [Row, Col]1..=1000 = read()
let pre = prefix(raisins)

var pay: [Row, Row, Col, Col]0..=4e9 = 0  // an optimum cuts each raisin O(log nm) times
for r1 in Row.rev:
    for r2 in r1..=Row.last!:
        for c1 in Col.rev:
            for c2 in c1..=Col.last!:
                if r1 == r2 and c1 == c2: continue
                var best = Int.MAX
                for r in r1..r2: best min= pay[r1, r, c1, c2] + pay[r.next!, r2, c1, c2]
                for c in c1..c2: best min= pay[r1, r2, c1, c] + pay[r1, r2, c.next!, c2]
                pay[r1, r2, c1, c2] = best + pre.fold(r1..=r2, c1..=c2)
print(pay[Row.first!, Row.last!, Col.first!, Col.last!])
