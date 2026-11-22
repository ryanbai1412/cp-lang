// ceoi19pp1: count pairs (i < j in a, k < l in b) with a[j] - a[i] = b[l] - b[k]: tally the
// differences of a, then look up every difference of b.
let (n, m) = read<(1..=2_000, 1..=2_000)>()
let a = read<[n]Int>()
let b = read<[m]Int>()
let seen = a.pairs().map(p => p.1 - p.0).tally()
print(b.pairs().map(p => seen.get(p.1 - p.0) ?? 0).sum())
