// Codeforces 2150 E2: Hidden Single (Hard Version, interactive).
// An array of length `2n - 1` holds every value in `1..=n` twice except one
// hidden single `v*` that appears once. Shuffling positions with a fixed seed
// breaks adversarial clustering so expected queries stay within the
// `2n + 2 log n` budget.
//
// Split `Pos = 0..2n - 1` at `mid = (2n - 1) / 2` and record each value's
// left-half weight `left_w[v]` in `0..=2` (its right-half weight is
// `2 - left_w[v]`):
//   - present in both halves -> `left_w[v] = 1`;
//   - absent on left -> `left_w[v] = 0`; absent on right -> `left_w[v] = 2`.
// Every normal value contributes its true count to each half while `v*`
// contributes 2 to its half despite occupying 1 slot, so the half whose weight
// sum exceeds its length contains `v*` (with weight 2). Bisecting each
// candidate in that half splits every normal pair across sub-halves, while
// `v*` narrows to a single position.

@!interactive
use std.rand.Rng

const SHUFFLE_SEED = 48
var rng = Rng.new(SHUFFLE_SEED)

@!multitest
let n: 1..=300 = read()
domain Val = 1..=n
domain Pos = 0..2 * n - 1

var order = List(1..2 * n)
rng.shuffle(mut order)
let perm = [Pos] p => order[p.ord]

fn has(v: Val, lo: Pos.bounds, hi: Pos.bounds) -> Bool:
    if lo == hi: return false
    print("?", v, hi.ord - lo.ord, perm[lo..hi])
    read<Int>() == 1

let mid = Pos.bounds.nth((2 * n - 1) / 2)
var left_w: [Val]0..=2 = 0
for v in Val:
    left_w[v] = if not has(v, Pos.bounds.first, mid):
        0
    elif has(v, mid, Pos.bounds.last):
        1
    else:
        2

let (lo, hi, only_here) = if left_w.sum() > mid.ord:
    (Pos.bounds.first, mid, [Val] v => left_w[v] == 2)
else:
    (mid, Pos.bounds.last, [Val] v => 2 - left_w[v] == 2)

fn is_single(v: Val, start: Pos.bounds, stop: Pos.bounds) -> Bool:
    var (l, r) = (start, stop)
    while r.ord - l.ord > 1:
        let m = Pos.bounds.nth((l.ord + r.ord) / 2)
        if not has(v, l, m): l = m
        elif not has(v, m, r): r = m
        else: return false
    true

for v in Val if only_here[v]:
    if is_single(v, lo, hi):
        print("!", v)
        break
