// AtCoder DP X Tower: stack blocks so that the weight above each block is at
// most its solidness s; maximise the total value. Some best tower has w + s
// growing downwards, so slide blocks under the tower in that order.
let n: 1..=1000 = read()
let blocks: [0..n](w: 1..=1e4, s: 1..=1e4, v: 1..=1e9) = read()

var best: [0..=20000]Int = 0            // best value of a tower of exactly this weight
for (w, s, v) in sorted(blocks, by: b => b.w + b.s):
    for above in (0..=s).rev: best[above + w] max= best[above] + v
print(max(best))
