// Codeforces 2178 C — First or Second
//
// One element `a[k]` is left unchosen (weight `0`), `a[0]` gets `+1` (when
// `k > 0`), each `a[1..k]` can freely take `+1` or `-1`, and every `a[k + 1..n]`
// gets `-1`. Measuring each weight relative to `-1` (subtracting `a.sum()` at
// the end):
// - `cur` holds the best prefix delta over `a[0..i]`: `a[0]` contributes `+2 * a[0]`
//   (from `-1` to `+1`), and each `x` in `a[1..i]` contributes `max(0, 2 * x)`
//   (keep `-1` or flip to `+1`).
// - Skipping `a[i]` changes its weight from `-1` to `0` (`+x`), giving delta `cur + x`.
// - Initializing `best = a[0]` covers `k = 0` (skip `a[0]`, all others `-1`).

@!multitest
let n: 2..=200_000 = read()
let a: [0..n]-10 ** 9..=10 ** 9 = read()

var best = a[0]
var cur = 2 * a[0]
for x in a[1..n]:
    best max= cur + x
    cur += max(0, 2 * x)
print(best - a.sum())
