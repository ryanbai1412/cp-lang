// Codeforces 2127 B — Hamiiid, Haaamid... Hamid?
//
// `Pos = 1..=n` is 1-based; its 0-based index in `s` is `p = x.ord` (`0 <= p < n`).
// Escaping left takes `left_escape = 1` day if no wall `'#'` lies in `0..p`, or
// `l + 2` days if the nearest left wall is at `l` (`l + 1` cells plus the exit
// step). Symmetrically, escaping right takes `right_escape = 1` if no wall lies
// in `p + 1..n`, or `n - r + 1` if the nearest right wall is at `r`.
//
// On day 1 the builder may place one new wall on an empty neighbour of `p` to
// maximize the runner's cheaper escape `min(left, right)`:
// - Placing at `p - 1` (empty iff `left_escape < p + 1`) raises the left escape
//   cost to `p + 1`, giving `min(p + 1, right_escape)`.
// - Placing at `p + 1` (empty iff `right_escape < n - p`) raises the right
//   escape cost to `n - p`, giving `min(left_escape, n - p)`.
// - Otherwise (the builder cannot raise either side's cost further) the answer
//   stays `min(left_escape, right_escape)`.

@!multitest
let n: 1..=200_000 = read()
domain Pos = 1..=n
let x: Pos = read()
let s: [0..n]Char = read()

let p = x.ord
let left_escape = if let l = (0..p).rev.first(i => s[i] == '#'): l + 2 else: 1
let right_escape = if let r = (p + 1..n).first(i => s[i] == '#'): n - r + 1 else: 1
var ans = min(left_escape, right_escape)
if left_escape < p + 1: ans max= min(p + 1, right_escape)
if right_escape < n - p: ans max= min(left_escape, n - p)
print(ans)
