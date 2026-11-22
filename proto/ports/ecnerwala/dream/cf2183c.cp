// Codeforces 2183 C — War Strategy
//
// Starting at base `k` on `1..=n` with `m_in` days, let `l = min(k - 1, n - k)`
// and `r = max(k - 1, n - k)` be the distances to the near and far ends.
//
// Occupying `a <= l` cells on the near side and `b <= r` cells on the far side
// takes `a + b` soldier-spawns plus `max(a, b) - 1` extra march days for the
// longer wave (the first march step happens on the same day as the first spawn,
// saving 1 day), for a total cost of `max(2a + b, a + 2b) - 1`. Letting
// `m = m_in + 1` absorbs that `- 1`, so the larger side binds: we need
// `max(2a + b, a + 2b) <= m` (i.e. `a + 2b <= m` when `a <= b`).
//
// Maximizing `a + b + 1` subject to `a <= l`, `b <= r`, and `max(2a + b, a + 2b) <= m`:
// - If `l + 2r <= m`: all `n = l + r + 1` cells are reachable.
// - Else if `3l <= m`: exhaust the near side (`a = l`), leaving
//   `b = (m - l) / 2` cells on the far side (`l + (m - l) / 2 + 1`).
// - Otherwise (`3l > m`): neither wall binds, so balance `a` and `b` around
//   `m / 3` to get `a + b = floor(2m / 3)` (`2 * m / 3 + 1` cells).

@!multitest
let (n, m_in, k): (1..=10 ** 9, 1..=10 ** 9, 1..=10 ** 9) = read()
let m = m_in + 1
let (l, r) = (min(k - 1, n - k), max(k - 1, n - k))
let ans = if l + 2 * r <= m: l + r + 1
elif 3 * l <= m: l + (m - l) / 2 + 1
else: 2 * m / 3 + 1
print(ans)
