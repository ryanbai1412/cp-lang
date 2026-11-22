// Codeforces 2261 E — Bracket Nesting Depth with k Swaps
//
// Scan the balanced bracket string `s[0..n]` maintaining `open` = indices of
// unmatched `'('` so far. After pushing index `i`, the prefix `0..=i` contains
// `i + 1 - open.len` already-matched characters (each matched pair costs 1 move
// to clear, i.e. 2 characters per move):
// - If `i + 1 - open.len <= 2 * k`: all matched pairs before `i` fit within `k`
//   moves, and the remaining moves can nest up to `k` more pairs around `open`,
//   reaching depth `k + open.len` (capped at `n / 2` globally, the max depth
//   of any length-`n` balanced string).
// - Otherwise: for `j in 0..open.len`, the non-stack characters inside
//   `[open[j]..=i]` number `f(j) = (i + 1 - open[j]) - (open.len - j)`. As `j`
//   increases, `open[j]` increases by at least 1 per step while `open.len - j`
//   decreases by 1, so `f(j)` is strictly decreasing in `j`, justifying
//   `bsearch_first` for the earliest `j` with `f(j) <= 2 * k` (depth `open.len - j`).

@!multitest
let (n, k): (1..=200_000, 0..=200_000) = read()
let s: [0..n]Char = read()

var open: List<Int> = []
var ans = 0
for (i, c) in s.items:
    if c == ')':
        open.pop()
        continue
    open.push(i)
    if i + 1 - open.len <= 2 * k:
        ans max= k + open.len
    else:
        let from = bsearch_first(0..open.len, j => (i + 1 - open[j]) - (open.len - j) <= 2 * k)!
        ans max= open.len - from
print(min(ans, n / 2))
