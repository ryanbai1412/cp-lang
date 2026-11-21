// AtCoder DP X Tower: stack blocks (weight w, solidness s, value v) so that
// the weight above each block is at most its s. Some best tower has w + s
// non-decreasing downwards, so add blocks in that order under the tower.
let n: 1..=1000 = read()
type Block = (w: 1..=1e4, s: 1..=1e4, v: 1..=1e9)
let blocks: [n]Block = read()
domain Load = 0..=20000

var best: [Load]Int = 0                 // best value of a tower weighing at most this
for b in blocks.sorted(by: b => b.w + b.s):
    for above in (0..=b.s).rev:
        best[Load(above + b.w)] max= best[Load(above)] + b.v
print(max(best[l] for l in Load))
