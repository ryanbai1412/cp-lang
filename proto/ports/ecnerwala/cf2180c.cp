// Codeforces 2180 C — XOR-Factorization
//
// Construct `k` integers `a[0..k]` in `0..=n` (`n <= 10^9 < 2^31`) with
// `XOR(a) == n` and maximum sum:
// - If `k` is odd, `k` copies of `n` have XOR `n` and achieve the upper bound `k * n`.
// - If `k` is even, build `a` bit by bit from `MAX_BIT = 30` down to `0`,
//   tracking `num_smaller`: the prefix `a[0..num_smaller]` is already strictly
//   `< n` (free on all lower bits), while `a[num_smaller..k]` still matches `n`.
//   - If `n.bit(b)` is 1: we need an odd number (`k - 1`) of `1`s at bit `b`.
//     If `num_smaller < k`, increment `num_smaller` (the newly released element
//     gets `0` at bit `b`); otherwise `a[k - 1]` gets `0`. In both cases
//     `num_smaller >= 1`, so `a[num_smaller - 1]` is in bounds and excluded.
//   - If `n.bit(b)` is 0: only the `num_smaller` free elements may take bit `b`,
//     and we must pick an even count to preserve XOR 0, so `0..(num_smaller / 2 * 2)`
//     receive `1 << b`.

const MAX_BIT = 30

@!multitest
let (n, k): (1..=10 ** 9, 1..=100_000) = read()
if k % 2 == 1:
    print(List.filled(k, n))
    return

var a: [0..k]Int = 0
var num_smaller = 0
for b in (0..=MAX_BIT).rev:
    if n.bit(b):
        if num_smaller < k: num_smaller += 1
        a[0..k] += 1 << b
        a[num_smaller - 1] -= 1 << b
    else:
        a[0..num_smaller / 2 * 2] += 1 << b
print(a)
