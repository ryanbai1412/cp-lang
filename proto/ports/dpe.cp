// cp: 0.1 1f59ac0+dirty
// AtCoder DP E Knapsack 2: weights are huge but values are small, so keep the
// lightest way to reach each total value.
let n: 1..=100 = read()
let cap: 1..=1e9 = read()
let items: [n](w: 1..=1e9, v: 1..=1000) = read()
domain Value = 0..=100000

var lightest: [Value]?Int = none
lightest[Value.first] = 0
for (w, v) in items:
    for total in Value.rev:
        if let rest = total - v and let base = lightest[rest]: lightest[total] min= base + w
print(max(t.label for t in Value if (lightest[t] ?? inf) <= cap))
