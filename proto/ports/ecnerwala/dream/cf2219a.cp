// Codeforces 2219 A — Grid L-Trominoes
//
// An `n x m` grid of unit squares (`n, m >= 1`) has `n * (m + 1) + m * (n + 1)
// = 2 * n * m + n + m` grid segments. Covering all segments with `p` single
// segments and `q` two-segment L-pieces requires `p + 2 * q = 2 * n * m + n + m`,
// equivalently `prod = 2 * (p + 2 * q) + 1 = (2 * n + 1) * (2 * m + 1)`.
//
// Since `n, m >= 1`, `2 * n + 1 >= 3`, so we search odd divisors `i` in
// `3..=isqrt(prod)` (giving `n <= m` in `O(sqrt(p + 2 * q))`, <= 4e4 steps for
// `p, q <= 10^9`). Each L-piece consumes one horizontal and one vertical
// segment, so `q` cannot exceed the smaller orientation count
// `min(n * (m + 1), m * (n + 1)) = n * m + n`; conversely, any `q <= n * m + n`
// can be placed greedily along grid corners.

@!multitest
let (p, q): (0..=10 ** 9, 0..=10 ** 9) = read()
let prod = (p + 2 * q) * 2 + 1
for i in 3..=isqrt(prod) by 2 if prod % i == 0:
    let (n, m) = (i / 2, prod / i / 2)
    if q <= n * m + n:
        print(n, m)
        return
print(-1)
