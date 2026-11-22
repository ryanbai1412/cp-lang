// Codeforces 2229 B — Pair Orientation Sum
//
// Orient each pair `(a[i], b[i])` as `(lo, hi)` with `lo <= hi`. Each pair
// contributes at least its larger entry `hi` to the total, and at most one
// pair can additionally contribute its smaller entry `lo`; choosing the pair
// with the largest `lo` maximizes the sum at `sum(hi) + max(lo)`.

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]1..=10 ** 9 = read()
let b: [0..n]1..=10 ** 9 = read()
let pairs = [(min(x, y), max(x, y)) for (x, y) in zip(a, b)]
print(sum(hi for (_, hi) in pairs) + max(lo for (lo, _) in pairs)!)
