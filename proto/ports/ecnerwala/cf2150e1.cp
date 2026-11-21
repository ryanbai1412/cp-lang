// Codeforces 2150 E1: Hidden Single (Easy Version, interactive).
// An array of length `2n - 1` holds every value in `1..=n` twice except one
// hidden single `v*` that appears once. Shuffling positions with a fixed seed
// breaks adversarial clustering so each halving drops ~half the active pairs
// within the `4n + 2 log n` query budget.
//
// Binary-search `[lo, hi)` (while `alive.len > 1`, `hi - lo >= 2` so `mid > lo`):
// every value in `Val` has known multiplicity `copies[v]` in `[lo, hi)` (`None`,
// `One`, or `Both`). Querying `left[v]` on `[lo, mid)` determines `right[v] =
// copies[v] - left[v]`:
//   - `Both`: absent on left -> `left = None`; else present on right -> `One`;
//     else `Both`.
//   - `One`: present on right -> `left = None`, else `One`.
// Every non-singleton contributes 2 units and `v*` contributes 1, so the half
// whose total weight exceeds its length contains `v*`.

@!interactive
use std.rand.Rng

enum Copies: None | One | Both

fn weight(c: Copies) -> Int = match c:
    Copies.None => 0
    Copies.One => 1
    Copies.Both => 2

fn complement(total: Copies, left: Copies) -> Copies = match weight(total) - weight(left):
    0 => Copies.None
    1 => Copies.One
    _ => Copies.Both

const SHUFFLE_SEED = 48
var rng = Rng.new(SHUFFLE_SEED)

@!multitest
let n: 1..=300 = read()
domain Val = 1..=n

var perm = List(1..2 * n)
rng.shuffle(mut perm)

fn has(v: Val, lo: Int, hi: Int) -> Bool:
    if lo == hi: return false
    print("?", v, hi - lo, perm[lo..hi])
    read<Int>() == 1

var copies: [Val]Copies = Copies.Both
var (lo, hi) = (0, 2 * n - 1)
var alive = [v for v in Val]

while alive.len > 1:
    let mid = (lo + hi) / 2
    var left: [Val]Copies = Copies.None
    for v in alive:
        left[v] = if not has(v, lo, mid):
            Copies.None
        elif has(v, mid, hi):
            Copies.One
        else:
            Copies.Both
    for v in Val if copies[v] == Copies.One:
        left[v] = if has(v, mid, hi): Copies.None else: Copies.One

    let left_weight = [weight(left[v]) for v in Val].sum()
    if left_weight > mid - lo:
        hi = mid
        copies = left
    else:
        lo = mid
        copies = [Val] v => complement(copies[v], left[v])
    alive = [v for v in Val if copies[v] == Copies.Both]

print("!", alive[0])
