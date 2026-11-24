// IOI 2009 P4 (Raisins), tuned: port.cp's recurrence filled bottom-up into one flat 4-byte table
// indexed by ((r1 n + r2) m + c1) m + c2, blocks before the blocks that contain them, instead of
// `@memo` with a seen flag per lookup and a four-index grid.
let n: 1..=50 = read()
let m: 1..=50 = read()
domain Row = 1..=n
domain Col = 1..=m
let raisins: [Row, Col]1..=1000 = read()
let pre = prefix(raisins)

let (row2, row1) = (m * m, n * m * m)
var pay: [n * n * m * m]0..=4e9 = 0  // an optimum cuts each raisin O(log nm) times
for r1 in Row.rev:
    for r2 in r1..=Row.last!:
        let rows = r1.ord * row1 + r2.ord * row2
        for c1 in Col.rev:
            for c2 in c1..=Col.last!:
                if r1 == r2 and c1 == c2: continue
                let cols = c1.ord * m + c2.ord
                var best = Int.MAX
                for r in r1.ord..r2.ord:
                    best min= pay[r1.ord * row1 + r * row2 + cols] + pay[(r + 1) * row1 + r2.ord * row2 + cols]
                for c in c1.ord..c2.ord:
                    best min= pay[rows + c1.ord * m + c] + pay[rows + (c + 1) * m + c2.ord]
                pay[rows + cols] = best + pre.fold(r1..=r2, c1..=c2)
print(pay[(n - 1) * row2 + m - 1])
