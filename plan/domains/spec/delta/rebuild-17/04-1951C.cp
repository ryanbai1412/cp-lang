// 1951C Ticket Hoarding: buy greedily on the cheapest days; each ticket bought raises later prices.
@!multitest
let n: 1..=3e5 = read()
let m: 1..=1e9 = read()
let k: 1..=1e9 = read()
let a = read<[n]1..=1e9>()
var left = k
var bought = 0
var cost = 0
for p in a.sorted():
    if left == 0: break
    let x = min(m, left)
    cost += x * (p + bought)
    bought += x
    left -= x
print(cost)
