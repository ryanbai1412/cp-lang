// ceoi17p2 Sure Bet: bet on the i best A odds and the j best B odds; the profit is
// min(sa[i], sb[j]) - i - j for the two prefix sums. For each i the best j sits where the B
// prefix sum first reaches the A prefix sum, so check the cut there and the one before it.
let n = read<1..=100_000>()
let odds = read<[n](a: F64, b: F64)>()
let sa = odds.map(o => o.a).sorted(by: desc).scan(+)
let sb = odds.map(o => o.b).sorted(by: desc).scan(+)
var best = 0.0
for i in 0..=n:
    let j = Int(sb.first_ge(sa[i]) ?? n)
    for k in [j, max(j - 1, 0)]:
        best max= min(sa[i], sb[k]) - F64(i + k)
print(best, precision: 4)
