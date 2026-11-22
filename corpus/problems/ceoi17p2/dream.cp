// ceoi17p2 Sure Bet: bet on the i best A odds and the j best B odds; the profit is
// min(A_i, B_j) - i - j for the two prefix sums. For each i the best j sits where the B prefix
// sum crosses the A prefix sum, so check the cut there and the one before it.
let n = read<1..=100_000>()
let odds = read<[n](a: F64, b: F64)>()
let sa = odds.map(o => o.a).sorted(by: desc).scan(+)
let sb = odds.map(o => o.b).sorted(by: desc).scan(+)
var best = 0.0
for i in sa.keys:
    let j = sb.keys.first_true(c => sb[c] >= sa[i]) ?? sb.keys.last!
    for k in [j, max(j - 1, 0)]:
        best max= min(sa[i], sb[k]) - (i + k)
print(best, precision: 4)
