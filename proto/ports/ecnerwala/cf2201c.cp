// Codeforces 2201 C — Valid Bracket-Subsequence Counting
//
// Scan the balanced bracket string `s` tracking prefix balance `pref` and four
// modular counts over prefixes seen so far:
// - `ways_any`: total non-empty subsequences of the prefix (`2^i - 1`).
// - `ways_open` / `ways_close`: valid removable subsequences whose last picked
//   character is `'('` / `')'`, where removing `'('` is only valid while the
//   remaining suffix never drops below balance 0 (`pref >= 2` after the step;
//   whenever `pref < 2`, all pending `ways_open` subsequences become invalid
//   and reset to `0`).
// - `ans`: total valid non-empty subsequences (`+ 1 + ways_any` on `'('`,
//   `+ 1 + ways_open + ways_close` on `')'`).

const MOD = 998_244_353
type Z = Mod<MOD>

@!multitest
let _n: 1..=200_000 = read()
let s: Str = read()

var (ways_any, ways_open, ways_close, ans): (Z, Z, Z, Z) = (0, 0, 0, 0)
var pref = 0
for c in s:
    ans += 1
    if c == '(':
        ans += ways_any
        ways_open += ways_open + ways_close + 1
        pref += 1
    else:
        ans += ways_open + ways_close
        ways_close += ways_open + ways_close + 1
        pref -= 1
    if pref < 2: ways_open = 0
    ways_any = ways_any * 2 + 1
print(ans)
