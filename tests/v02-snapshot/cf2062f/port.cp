// Source: Codeforces 2062F (Traveling Salescat), https://codeforces.com/contest/2062/problem/F
// Problem: n <= 3000 cities with 0 <= a_i, b_i <= 1e9; the road i-j has length
//   max(a_i + b_j, b_i + a_j). For each k = 2..n print the least cost of a simple path through
//   exactly k cities. Multitest, sum of n^2 <= 9e6.
// Solution: with x = (a + b) / 2 and y = (a - b) / 2 the road costs x_i + x_j + |y_i - y_j|.
//   A path pays x twice per inner city and once per end; its least y cost is twice its y-span
//   less the y gap of its two ends. So in y order each chosen city pays by role: the lowest 2b
//   (b if also an end), a middle a + b, an end a or b by side, the highest 2a (a if also an end).
//   DP over cities in y order on (cities chosen, roles filled: 5 stages) with 9 role transitions:
//   O(n^2) per test.
@!multitest
let n: 2..=3000 = read()
domain Stage = 0..5
let (start, low, one_end, two_ends, done) = (Stage(0), Stage(1), Stage(2), Stage(3), Stage(4))

fn roles(a: Int, b: Int) -> List<(Stage, Stage, Int)> = [
    (start, low, 2 * b),
    (start, one_end, b),
    (low, low, a + b),
    (low, one_end, a),
    (one_end, one_end, a + b),
    (one_end, two_ends, b),
    (one_end, done, a),
    (two_ends, two_ends, a + b),
    (two_ends, done, 2 * a),
]

var cost: [0..=n, Stage]?Int = None
cost[0, start] = 0
for (i, (a, b)) in read<[n](a: 0..=1e9, b: 0..=1e9)>().sorted(by: c => c.a - c.b).items:
    for k in (0..=i).rev, (from, to, w) in roles(a, b):
        cost[k + 1, to] min= cost[k, from] + w
print([cost[k, done]! for k in 2..=n])
