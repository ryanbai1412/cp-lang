// Codeforces 2127 B — Hamiiid, Haaamid... Hamid?
//
// Starting at 1-based position `x` in `Pos = 1..=n`, escaping left takes
// `left_escape = 1` day if no wall `'#'` lies to the left of `x`, or `l + 2`
// days if the nearest left wall is at 0-based index `l` (`l + 1` cells plus
// the exit step). Symmetrically, escaping right takes `right_escape = 1` if no
// wall lies to the right, or `n - r + 1` if the nearest right wall is at `r`.
//
// On day 1 the builder may place one new wall on either empty neighbour of `x`
// to maximize the runner's cheaper escape `min(left, right)`:
// - Placing at `x - 1` (valid iff `left_escape < x.ord`) raises the left cost
//   to `x.ord`, giving `min(x.ord, right_escape)`.
// - Placing at `x + 1` (valid iff `right_escape < n - x.ord + 1`) raises the
//   right cost to `n - x.ord + 1`, giving `min(left_escape, n - x.ord + 1)`.
// - Otherwise (both neighbours already walled) the cost stays
//   `min(left_escape, right_escape)`.

@!multitest
let n: 1..=200_000 = read()
domain Pos = 1..=n
let x: Pos = read()
let s: Str = read()

let left_escape = if let l = s[0..x.ord].rfind('#'): l + 2 else: 1
let right_escape = if let r = (x.ord + 1..n).first(i => s[i] == '#'): n - r + 1 else: 1
var ans = min(left_escape, right_escape)
if left_escape < x.ord + 1: ans max= min(x.ord + 1, right_escape)
if right_escape < n - x.ord: ans max= min(left_escape, n - x.ord)
print(ans)
