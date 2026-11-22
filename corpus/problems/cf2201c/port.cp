// Codeforces 2201 C — Valid Bracket-Subsequence Counting
//
// A non-empty subsequence `T` of the balanced bracket string `s` is valid iff
// cyclically shifting `T`'s characters one step right (moving the last picked
// character `T_last` to the first picked position `T_first`) keeps `s` balanced
// (every prefix balance stays `>= 0`).
//
// Scan `s` tracking its prefix balance `pref` (after index `i`) and two counts
// over valid subsequences of `s[0..i]` that could still be closed by a later
// `')'`: `ways_open` (ending in `'('`) and `ways_close` (ending in `')'`).
// - At `s[i] == '('`: moving a trailing `'('` to `T_first` never lowers any
//   prefix balance, so every subsequence of `s[0..i]` (empty or non-empty,
//   `Z(2) ** i` in total) yields a valid subsequence ending at `i`, and
//   `ways_open` grows by `1 + ways_open + ways_close`.
// - At `s[i] == ')'`: moving a trailing `')'` onto an earlier `'('` lowers the
//   prefix balance by `2` from that `'('` onward, so only the `1 + ways_open +
//   ways_close` subsequences whose pending `'('` never crossed a `pref < 2`
//   dip remain valid; both `ans` and `ways_close` grow by `1 + ways_open +
//   ways_close`.
// - Whenever `pref < 2` after step `i`, dropping `pref` by `2` would go
//   negative, so no pending subsequence in `ways_open` can ever accept a later
//   `')'`, and `ways_open` resets to `0`.

const MOD = 998_244_353
type Z = Mod<MOD>

@!multitest
let n: 1..=200_000 = read()
let s: [0..n]Char = read()

var (ways_open, ways_close, ans): (Z, Z, Z) = (0, 0, 0)
var pref = 0
for (i, c) in s.items:
    if c == '(':
        ans += Z(2) ** i
        ways_open += ways_open + ways_close + 1
        pref += 1
    else:
        let add = ways_open + ways_close + 1
        ans += add
        ways_close += add
        pref -= 1
    if pref < 2: ways_open = 0
print(ans)
