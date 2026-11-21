// Codeforces 2006 D: the best order alternates large and small values, so a window of length L needs,
// for every i <= sqrt(k), the values too big to sit beside anything above i matched by values <= i,
// up to floor(L / 2) of each, and one change fixes two mismatches. A value b > sqrt(k) matters only
// through k / b, so both sides are histograms over 1..=sqrt(k), kept per window by Mo's algorithm.
@!multitest
let (n, q, k) = read<(2..=1e5, 1..=1e5, 1..=1e6)>()
domain P = 1..=n
domain Level = 1..=k.isqrt
enum Side: Small(Level), Big(Level)
let side = read<[P]1..=k>().map(b => if b <= Level.last: Small(Level(b)) else: Big(Level(k / b)))
var small, big: [Level]Int = 0
small[Level.first] += 1
let days = read<[q]P.closed_ranges>()
print(days.mo(
    step: (i, d) => match side[i]: Small(l) => small[l] += d, Big(l) => big[l] += d,
    answer: w => zip(big.prefix_sums(), small.prefix_sums()).max(
        (need, have) => (need - have + (w.len + 1 > need + have).int + 1) / 2) ?? 0,
))
