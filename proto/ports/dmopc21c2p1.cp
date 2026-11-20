// cp: 0.1 0268fdb
// DMOPC '21 Contest 2 P1: pick a water level L; pay a per unit of level and b
// per unit that each post stands above it. The cost is piecewise linear in L,
// so the best level is 0 or the height of some post.
let n: 1..=1e6 = read()
let a: 1..=1e6 = read()
let b: 1..=1e6 = read()
domain Rank = 1..=n
let heights: [Rank]0..=1e6 = read()
let h = sorted(heights)

var best = b * sum(h)                   // level 0
var taller = 0                          // posts above the current level
var taller_sum = 0
for i in Rank.rev:
    best min= a * h[i] + b * (taller_sum - taller * h[i])
    taller += 1
    taller_sum += h[i]
print(best)
