// Codeforces 2062 F: with x = (a + b) / 2 and y = (a - b) / 2, the road i-j costs x_i + x_j + |y_i - y_j|.
// A path pays x twice per inner city and once per end, and its y cost is twice its y-span less what the
// two ends cut off; in y order each chosen city takes one role: the lowest (2b, or b if it is also an end),
// a middle (a + b), an end (a or b by side) or the highest (2a, or a if an end). The DP over cities in
// y order tracks how many are chosen and which roles are filled.
@!multitest
let n: 2..=3000 = read()
enum Stage: Start, Low, OneEnd, TwoEnds, Done

fn roles(a: Int, b: Int) -> List<(Stage, Stage, Int)> = [
    (Start, Low, 2 * b),
    (Start, OneEnd, b),
    (Low, Low, a + b),
    (Low, OneEnd, a),
    (OneEnd, OneEnd, a + b),
    (OneEnd, TwoEnds, b),
    (OneEnd, Done, a),
    (TwoEnds, TwoEnds, a + b),
    (TwoEnds, Done, 2 * a),
]

var cost: [0..=n, Stage]?Int = none
cost[0, Start] = 0
for (i, (a, b)) in read<[n](a: 0..=1e9, b: 0..=1e9)>().sorted(by: c => c.a - c.b).items:
    for k in (0..=i).rev, (from, to, w) in roles(a, b):
        cost[k + 1, to] min= cost[k, from] + w
print([cost[k, Done]! for k in 2..=n])
