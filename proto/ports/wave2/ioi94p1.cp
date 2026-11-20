// IOI 1994 The Triangle: the largest sum on a path from the top of a number
// triangle to its base, each step going down-left or down-right.
let n: 1..=100 = read()
var best: List<Int> = []                // best path sum down to each cell of the row above
for r in 1..=n:
    let row: [0..r]0..=99 = read()
    best = [row[c] + max(best.get(c - 1) ?? 0, best.get(c) ?? 0) for c in 0..r]
print(max(best))
