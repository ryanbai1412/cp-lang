let n: 1..=2e5 = read()
let h: [0..n]1..=1e4 = read()
var cost: [0..n]Int = 0
for i in 1..n:
    cost[i] = cost[i - 1] + abs(h[i] - h[i - 1])
    if i >= 2: cost[i] min= cost[i - 2] + abs(h[i] - h[i - 2])
print(cost[n - 1])
