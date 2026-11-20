// DMOPC '21 Contest 2 P4 — Run-Length 3-State Enum DP (CP Lang v2 spec)
// original: corpus/submissions/3949962.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Group equal adjacent elements with `runs(a)`, then run a 3-state rolling DP
// indexed by a unit `enum Carry` (`Neutral | Push | Pull`) over `?Int` so no
// `INF` sentinel is needed.

let (n, k): (1..=1e6, 0..=1e6) = read()
domain I = 0..n
let a: [I]Int = read()

let stride = 2 * k + 1
// FRICTION UD11: Ceiling division `(x + d - 1) / d` is written by hand because
// the v2 math prelude (§10) lists `abs, min, max, gcd, lcm, isqrt` but omits
// `ceil_div` (whereas Euclidean `/` gives floor division only).
fn blocks(x: Int) -> Int:
    if x <= 0: 0 else: (x + stride - 1) / stride

enum Carry = Neutral | Push | Pull

var dp: [Carry]?Int = none
dp[Neutral] = 0
var prev_val = 0

for (x, cnt) in runs(a):
    var ndp: [Carry]?Int = none

    fn relax(base: ?Int, rem: Int):
        if let d = base:
            ndp[Neutral] min= d + blocks(rem)
            ndp[Push] min= d + blocks(rem + 1)
            ndp[Pull] min= d + blocks(rem - k - 1)

    relax(dp[Neutral], cnt)
    if prev_val > x:
        relax(dp[Push], cnt - k - 1)
    else:
        relax(dp[Pull], cnt + 1)

    prev_val = x
    dp = ndp

print(dp[Neutral]!)
