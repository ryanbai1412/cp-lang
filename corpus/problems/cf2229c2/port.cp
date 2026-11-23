// Codeforces 2229 C2 — Prefix Sign-Flip Pivot Construction
//
// Each operation flips the signs of a prefix `0..=p` while requiring the
// rightmost flipped element `a[p]` to be non-negative before the final flip.
// Choosing a single pivot `idx` with `a[idx] >= 0` lets us turn every negative
// entry `a[j] < 0` (`j < idx`) positive while sacrificing only `a[idx]`, for a
// net gain of `cur_delta - 2 * a[idx]` where `cur_delta = sum_{j < idx, a[j] < 0} (-2 * a[j])`.
//
// If the best pivot gives a positive gain, sweep `i` from `idx - 1` down to `0`
// toggling `flip` whenever `(a[i] > 0) != flip` (recording `i + 1`), then
// append `idx + 1` to finish all prefix flips.

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]-10 ** 9..=10 ** 9 = read()
var best_delta = 0
var best_idx: ?Int = None
var cur_delta = 0
for (i, v) in a.items:
    if v >= 0:
        if cur_delta - 2 * v > best_delta:
            best_delta = cur_delta - 2 * v
            best_idx = i
    else:
        cur_delta -= 2 * v

var ops: List<Int> = []
if let idx = best_idx:
    var flip = false
    for i in (0..idx).rev if (a[i] > 0) != flip:
        ops.push(i + 1)
        flip = not flip
    ops.push(idx + 1)
print(ops.len)
print(ops)
