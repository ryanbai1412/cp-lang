// IOI 2017 Day 1 P2, Wiring (buildable today-form)
// original: corpus/submissions/5724153.cpp (grader function min_total_length)
// stdin: n m, the n red positions, the m blue positions (each sorted).
let n: 1..=100000 = read()
let m: 1..=100000 = read()
let red: [0..n]Int = read()
let blue: [0..m]Int = read()

// merge both colours by position; sign +1 for red, -1 for blue
var at: List<Int> = []
var sign: List<Int> = []
var i = 0
var j = 0
while i < n or j < m:
    if j == m or (i < n and red[i] < blue[j]):
        at.push(red[i])
        sign.push(1)
        i += 1
    else:
        at.push(blue[j])
        sign.push(-1)
        j += 1
let k = n + m

// near[p]: distance from point p to the closest point of the other colour
const FAR = 4e18
var near: List<Int> = []
var seen_red = -FAR
var seen_blue = -FAR
for p in 0..k:
    near.push(at[p] - (if sign[p] == 1: seen_blue else: seen_red))
    if sign[p] == 1: seen_red = at[p]
    else: seen_blue = at[p]
seen_red = FAR
seen_blue = FAR
for p in (0..k).rev:
    near[p] min= (if sign[p] == 1: seen_blue else: seen_red) - at[p]
    if sign[p] == 1: seen_red = at[p]
    else: seen_blue = at[p]

// cost[p]: cheapest wiring of the first p points. Either point p-1 takes its
// own wire to the nearest other colour, or the block since the last prefix
// with the same red-minus-blue balance is matched like brackets, which costs
// exactly |sum of signed positions| over the block.
var cost: List<Int> = []
cost.push(0)
var last: [0..=2 * k]Int = -1  // balance + k -> latest prefix with that balance
var bal = k
var signed = 0
var signed_at: List<Int> = []
signed_at.push(0)
last[bal] = 0
for p in 0..k:
    bal += sign[p]
    signed += sign[p] * at[p]
    var best = cost[p] + near[p]
    if last[bal] != -1:
        best min= cost[last[bal]] + abs(signed - signed_at[last[bal]])
    cost.push(best)
    signed_at.push(signed)
    last[bal] = p + 1
print(cost[k])
