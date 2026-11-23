// 1729D Friends and the Restaurant
// in: t; n; x_1..x_n (spend); y_1..y_n (budget). out: per test, most groups of >= 2 friends whose spend <= budget.
@!multitest
let n = read<2..=1e5>()
let x = read<[n]1..=1e9>()
let y = read<[n]1..=1e9>()
let slack = zip(x, y).map((spend, budget) => budget - spend).sorted()
var (i, j) = (0, n - 1)
var groups = 0
while i < j:
    if slack[i] + slack[j] >= 0:
        groups += 1
        j -= 1
    i += 1
print(groups)
