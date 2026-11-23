// Codeforces 2178 B — Impost or Sus (`3 <= s.len <= 200_000`)
//
// Every `'u'` must have `'s'` on both sides (so the first and last characters
// are `'s'`, and no two `'u'`s are adjacent). Track `dp[Last]` = minimum edits
// on the processed prefix ending with `'u'` (`Last.U`) or `'s'` (`Last.S`):
// - Start with `dp[U] = 0, dp[S] = none` so the first character cannot stay `'u'`.
// - On `'s'`: keeping `'s'` costs 0 from either state (`dp[S] = m`), and
//   flipping `'s'` to `'u'` is never needed (`dp[U] = none`).
// - On `'u'`: keeping `'u'` requires a preceding `'s'` (`dp[U] = dp[S]`), while
//   flipping to `'s'` costs `+1` from either state (`dp[S] = 1 + m`).
// After the first character `dp[S]` is always finite (`s.len >= 3`), so
// `dp[Last.S]!` is guaranteed present and enforces that `s` ends with `'s'`.

enum Last:
    U
    S

@!multitest
let s: Str = read()
var dp: [Last]?Int = None
dp[Last.U] = 0
for c in s:
    let m = min(dp[Last.U], dp[Last.S])!
    if c == 's':
        dp[Last.U] = None
        dp[Last.S] = m
    else:
        dp[Last.U] = dp[Last.S]
        dp[Last.S] = 1 + m
print(dp[Last.S]!)
