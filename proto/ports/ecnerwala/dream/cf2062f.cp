// Codeforces 2062 F: with x = (a + b) / 2 and y = (a - b) / 2, the road i-j costs x_i + x_j + |y_i - y_j|.
// A path pays x twice per inner city and once per end, and its y cost is twice its y-span less what the
// two ends cut off; in y order each chosen city takes one role: the lowest (2b, or b if it is also an end),
// a middle (a + b), an end (a or b by side) or the highest (2a, or a if an end). The DP over cities in
// y order tracks how many are chosen and which roles are filled.
@!multitest
let n = read<2..=3000>()
enum Stage: start, low, one_end, two_ends, done
let roles = (a: Int, b: Int) => [
    (start, low, 2 * b), (start, one_end, b),
    (low, low, a + b), (low, one_end, a),
    (one_end, one_end, a + b), (one_end, two_ends, b), (one_end, done, a),
    (two_ends, two_ends, a + b), (two_ends, done, 2 * a),
]
var cost: [0..=n, Stage]?Int = none
cost[0, start] = 0
for (i, (a, b)) in read<[n](a: 0..=1e9, b: 0..=1e9)>().sorted(by: c => c.a - c.b).indexed:
    for k in (0..=i).rev, (from, to, w) in roles(a, b):
        cost[k + 1, to] min= cost[k, from] + w
print(cost[2..=n, done]!)
