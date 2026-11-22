// Codeforces 2147 B — Multiple Construction
//
// Place each integer `1..=n` twice in an array of length `2n` so the distance
// between the two copies of `x` is divisible by `x`. In
// `[n - 1, ..., 1, n, 1, ..., n - 1, n]`:
// - Each `x in 1..n` sits at distance `x` on either side of the first `n`, so
//   its two copies are `2 * x` apart (divisible by `x`).
// - The two copies of `n` sit at indices `n - 1` and `2n - 1`, distance `n` apart.

@!multitest
let n: 1..=200_000 = read()
let desc = (1..n).rev
print(desc + [n] + desc.rev + [n])
