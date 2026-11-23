// Codeforces 2150 D: Attraction Theory.
// Valid final configurations occupy a contiguous interval [L, R] of length
// `l` in 1..=n, with 1 base person per cell plus `extras = n - l` extra people
// placed as `k = extras / 2` pairs across the `l` cells and 0, 1, or 2
// singletons on the endpoints:
//   - `place_ways(b, m) = C(b - 1 + m, m)` counts splits of `m` pairs into `b`
//     cells (stars and bars).
//   - `place_weight(b, m) = (m / b) * C(b - 1 + m, m) = C(b - 1 + m, b)` sums
//     one cell's pair count over all splits (0 when `m <= 0`).
// For length `l >= 2`, decomposing by endpoint singletons gives each cell's
// interior weight `mid_w` and each endpoint's extra `end_bonus`:
//   - odd `extras = 2k + 1`: 1 singleton on L or R (2 symmetric choices) and
//     `k` pairs (2 people each) => `mid_w = 2 * ways(l, k) + 4 * weight(l, k)`,
//     `end_bonus = ways(l, k)`.
//   - even `extras = 2k`: either (0 singletons, `k` pairs) or (both ends get a
//     singleton, `k - 1` pairs) => `mid_w = ways(l, k) + ways(l, k - 1) +
//     2 * (weight(l, k) + weight(l, k - 1))`, `end_bonus = ways(l, k - 1)`.
// Across all `n - l + 1` intervals of length `l`, cell `c` is an interior cell
// in `min(c, l - 2, n - l + 1, n - 1 - c)` intervals — a trapezoid of height
// `cap = min(l - 2, n - l + 1)` summed in O(1) via `pref` and `pref_i`.

type Z = Mod<998_244_353>
let comb = Comb<Z>()

fn place_ways(bins: Int, pairs: Int) -> Z = comb.choose(bins - 1 + pairs, pairs)
fn place_weight(bins: Int, pairs: Int) -> Z =
    if pairs <= 0: Z(0) else: comb.choose(bins - 1 + pairs, bins)

// Per-cell contribution of a single (singleton_choices, pairs) configuration family.
fn family_mid(bins: Int, pairs: Int) -> Z =
    place_ways(bins, pairs) + place_weight(bins, pairs) * 2

@!multitest
let n: 1..=200_000 = read()
domain Cell = 0..n
let raw: [Cell]0..=1_000_000_000 = read()
let a = [Cell] c => Z(raw[c])

let pref = prefix(a)
let pref_i = prefix([Cell] c => a[c] * c.ord)

// Sums a[c] * min(c.ord, cap, n - 1 - c.ord) over all c in Cell.
fn trapezoid_sum(cap: Int) -> Z:
    if cap == 0: return Z(0)
    let lo = Cell(cap)
    let hi = Cell(n - cap)
    let ramp_up = pref_i.fold(Cell.first!..lo)
    let plateau = pref.fold(lo..hi) * cap
    let ramp_down = pref.fold(hi..=Cell.last!) * Cell.last!.ord - pref_i.fold(hi..=Cell.last!)
    ramp_up + plateau + ramp_down

var ans = pref.fold(Cell.first!..=Cell.last!) * n
for l in 2..=n:
    let extras = n - l
    let k = extras / 2
    let (mid_w, end_bonus) = if extras % 2 == 1:
        (family_mid(l, k) * 2, place_ways(l, k))
    else:
        (family_mid(l, k) + family_mid(l, k - 1), place_ways(l, k - 1))
    let edge_w = mid_w + end_bonus

    let ends_sum = pref.fold(Cell.first!..=Cell(n - l)) + pref.fold(Cell(l - 1)..=Cell.last!)
    let cap = min(l - 2, n - l + 1)
    ans += edge_w * ends_sum + mid_w * trapezoid_sum(cap)

print(ans)
