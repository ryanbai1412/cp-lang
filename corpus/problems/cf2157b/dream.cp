// Codeforces 2157 B — Expansion Plan 2
//
// Each `'8'` expansion covers 1 unit simultaneously on both `|x|` and `|y|`
// (Chebyshev), reducing the remaining distance on each axis to
// `max(0, abs(x0) - c8)` and `max(0, abs(y0) - c8)`. Each `'4'` expansion
// covers 1 unit on a single axis (Manhattan), so `(x0, y0)` is reachable iff
// the sum of remaining axial distances is at most `c4 = n - c8`.

@!multitest
let (n, x0, y0): (1..=200_000, -10 ** 9..=10 ** 9, -10 ** 9..=10 ** 9) = read()
let s: [0..n]Char = read()
let c8 = s.count(c => c == '8')
let c4 = n - c8
print(if max(0, abs(x0) - c8) + max(0, abs(y0) - c8) <= c4: "YES" else: "NO")
