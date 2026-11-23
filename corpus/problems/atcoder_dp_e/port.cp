// AtCoder DP E Knapsack 2: weights are huge but values are small, so keep the
// lightest way to reach each total value.
let (n, cap): (1..=100, 1..=1e9) = read()
let items: [0..n](w: 1..=1e9, v: 1..=1000) = read()

let top = sum(v for (_, v) in items)
var lightest: [0..=top]?Int = None
lightest[0] = 0
for (w, v) in items:
    for total in (v..=top).rev: lightest[total] min= lightest[total - v] + w
print([total for total in 0..=top if lightest[total] != None and lightest[total]! <= cap].max()!)
