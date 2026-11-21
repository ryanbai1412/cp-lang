// Codeforces 2150 E2: Hidden Single (Hard Version) (dream).
// An array of length `2n - 1` holds every value in `1..=n` twice except one
// hidden single `v*` that appears once. `has(v, l, r)` queries whether `v`
// occurs in the shuffled position slice `perm[l..r]`.
// Split `0..2n - 1` at midpoint `m` and assign each value a 2-unit weight:
//   - present in both halves -> 1 to left, 1 to right;
//   - present in only one half -> 2 to that half (0 to the other).
// Every normal value contributes its true count to each half, while `v*`
// contributes 2 to its half despite occupying only 1 slot. Thus the half whose
// weight sum exceeds its length contains `v*` (with weight 2).
// For each candidate `v` with weight 2 in that half, `is_single` bisects the
// interval: a normal value eventually splits across both sub-halves (false),
// whereas `v*` narrows all the way down to a single index (true).
@!interactive
use std.rand.Rng

var rng = Rng.new(48)

@!multitest
let n: 1..=300 = read()
domain Val = 1..=n

var perm = List(1..2 * n)
rng.shuffle(mut perm)

fn has(v: Val, lo: Int, hi: Int) -> Bool:
    if lo == hi: return false
    print("?", v, hi - lo, perm[lo..hi])
    read<Int>() == 1

let mid = (2 * n - 1) / 2
var weight_l: [Val]Int = 0
var weight_r: [Val]Int = 0
for v in Val:
    if not has(v, 0, mid):
        weight_r[v] = 2
    elif has(v, mid, 2 * n - 1):
        weight_l[v] = 1
        weight_r[v] = 1
    else:
        weight_l[v] = 2

let (lo, hi, only_here) = if weight_l.sum() > mid:
    (0, mid, [Val] v => weight_l[v] == 2)
else:
    (mid, 2 * n - 1, [Val] v => weight_r[v] == 2)

fn is_single(v: Val, start: Int, stop: Int) -> Bool:
    var (l, r) = (start, stop)
    while r - l > 1:
        let m = (l + r) / 2
        if not has(v, l, m): l = m
        elif not has(v, m, r): r = m
        else: return false
    true

let ans = [v for v in Val if only_here[v]].first(v => is_single(v, lo, hi))!
print("!", ans)
