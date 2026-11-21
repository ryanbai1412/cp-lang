// Codeforces 2150 D: Attraction Theory.
// Final configurations occupy a contiguous interval of length `l` in 1..=n, with
// `extras = n - l` extra people distributed in pairs (`k = extras / 2`) across the
// `l` cells. `place_ways(bins, balls)` counts nonnegative splits of `balls` into
// `bins` cells, and `place_weight(bins, balls)` sums one cell's occupancy over all
// splits (0 when `balls <= 0` because no cell can receive a ball).
// For each length `l >= 2`, every interior cell contributes `mid_w` and each
// endpoint contributes `edge_w = mid_w + end_bonus`, determined by the parity of
// `extras`. Across all `n - l + 1` intervals of length `l`, cell `i` is an interior
// cell in `min(i, l - 2, n - l + 1, n - 1 - i)` intervals — a trapezoid of height
// `cap = min(l - 2, n - l + 1)` evaluated in O(1) via `pref` and `pref_i`.
use std.prefix.prefix1

type Z = Mod<998_244_353>
let comb = Comb<Z>()

fn place_ways(bins: Int, balls: Int) -> Z = comb.choose(bins - 1 + balls, balls)
fn place_weight(bins: Int, balls: Int) -> Z = if balls <= 0: Z(0) else: comb.choose(
    bins - 1 + balls,
    bins,
)

@!multitest
let n: 1..=200_000 = read()
let raw: [0..n]0..=1_000_000_000 = read()
let a = [0..n] i => Z(raw[i])

let pref = prefix1(a)
let pref_i = prefix1([0..n] i => a[i] * i)

var ans = pref.sum(0..n) * n
for l in 2..=n:
    let extras = n - l
    let k = extras / 2
    let (mid_w, end_bonus) = if extras % 2 == 1:
        (place_ways(l, k) * 2 + place_weight(l, k) * 4, place_ways(l, k))
    else:
        (
            place_ways(l, k) + place_ways(l, k - 1) + (
                place_weight(l, k) + place_weight(l, k - 1)
            ) * 2,
            place_ways(l, k - 1),
        )
    let edge_w = mid_w + end_bonus

    let ends_sum = pref.sum(0..n - l + 1) + pref.sum(l - 1..n)
    let cap = min(l - 2, n - l + 1)
    let ramp_up = pref_i.sum(0..cap)
    let plateau = pref.sum(cap..n - cap) * cap
    let ramp_down = pref.sum(n - cap..n) * (n - 1) - pref_i.sum(n - cap..n)
    ans += edge_w * ends_sum + mid_w * (ramp_up + plateau + ramp_down)

print(ans)
