// cp: 0.1 1f59ac0+dirty
// IOI 1994 The Triangle: the largest sum on a path from the top of a number
// triangle to its base, each step going down-left or down-right.
let n: 1..=100 = read()
var best: List<Int> = []                // best path sum ending at each cell of the row above
for r in 1..=n:
    var row: List<Int> = []
    for c in 0..r:
        let x: 0..=99 = read()
        let left = if c > 0: best[c - 1] else: 0
        let right = if c < r - 1: best[c] else: 0
        row.push(x + max(left, right))
    best = row
print(max(b for b in best))
