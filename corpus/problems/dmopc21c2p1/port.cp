// DMOPC '21 Contest 2 P1: pick a water level L; pay a per unit of level and b
// per unit that each post stands above it. The cost is piecewise linear in L,
// so the best level is 0 or the height of some post.
let (n, a, b): (1..=1e6, 1..=1e6, 1..=1e6) = read()
let heights: [0..n]0..=1e6 = read()

var best = b * heights.sum()             // level 0
var taller = 0                          // posts above the current level
var taller_sum = 0
for h in heights.sorted().rev:
    best min= a * h + b * (taller_sum - taller * h)
    taller += 1
    taller_sum += h
print(best)
