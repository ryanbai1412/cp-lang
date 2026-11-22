// Codeforces 2183 C — Maximum Reachable Positions
//
// Starting at `k` on 1..=n with `m_in` steps, let `l = min(k - 1, n - k)` and
// `r = max(k - 1, n - k)` be the distances to the near and far ends.
//
// Reaching `a <= l` cells on the near side and `b <= r` cells on the far side
// takes `min(2a + b, a + 2b) - 1` steps (since the first cell is reached on
// step 1 without returning), so with `m = m_in + 1` we need `a + 2b <= m`
// (for `a <= b`) or `2a + b <= m` (for `a >= b`). Maximizing `a + b + 1`:
// - If `l + 2r <= m`: all `n = l + r + 1` cells are reachable.
// - Else if `3l <= m`: exhaust the near side (`a = l`), leaving `(m - l) / 2`
//   cells on the far side (`l + (m - l) / 2 + 1`).
// - Otherwise (`3l > m`): split evenly with `a + b = floor(2m / 3)` for
//   `2 * m / 3 + 1` cells.

@!multitest
let (n, m_in, k): (1..=10 ** 9, 1..=10 ** 9, 1..=10 ** 9) = read()
let m = m_in + 1
let (l, r) = (min(k - 1, n - k), max(k - 1, n - k))
let ans = if l + 2 * r <= m: l + r + 1
elif 3 * l <= m: l + (m - l) / 2 + 1
else: 2 * m / 3 + 1
print(ans)
