@!multitest
let n: 2..=3000 = read()
let cities = read<[n](a: 0..=1e9, b: 0..=1e9)>().sorted(by: c => c.a - c.b)

var cost: [n + 1][5]?Int = none
cost[0][0] = 0
for i in 0..n:
    let (a, b) = cities[i]
    let roles = [
        (0, 1, 2 * b),
        (0, 2, b),
        (1, 1, a + b),
        (1, 2, a),
        (2, 2, a + b),
        (2, 3, b),
        (2, 4, a),
        (3, 3, a + b),
        (3, 4, 2 * a),
    ]
    for k in (0..=i).rev, (from, to, w) in roles:
        cost[k + 1][to] min= cost[k][from] + w
print([cost[k][4]! for k in 2..=n])
