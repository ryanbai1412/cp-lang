// Codeforces 2084 A — Max and Mod
//
// We need a permutation `p[1..=n]` such that `max(p[i - 1], p[i]) % i == i - 1`
// for all `2 <= i <= n`. If `n` is placed at `pos` with `1..pos` before it and
// `pos..n` after it (`[1, ..., pos - 1, n, pos, ..., n - 1]`), every position
// except `pos` and `pos + 1` has `max(p[j - 1], p[j]) == j - 1`, while `pos`
// and `pos + 1` have max `n`, requiring `n % pos == pos - 1` and
// `n % (pos + 1) == pos`, i.e., both `pos` and `pos + 1` divide `n + 1`.
// When `n` is odd, `pos = 1` satisfies this (`[n, 1, 2, ..., n - 1]`); when `n`
// is even, `n + 1` is odd so no consecutive pair divides it and no solution exists.

@!multitest
let n: 2..=100 = read()
if let pos = first(i for i in 1..=n if (n + 1) % i == 0 and (n + 1) % (i + 1) == 0):
    print([if j == pos: n elif j < pos: j else: j - 1 for j in 1..=n])
else:
    print(-1)
