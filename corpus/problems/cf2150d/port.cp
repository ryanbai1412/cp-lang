// Source: Codeforces 2150D (Attraction Theory), https://codeforces.com/contest/2150/problem/D
// limits: 2000 ms, 256 MB
// Problem: n people stand at 1..n. Placing an attraction at x moves everyone one step toward
//   x. Over all distinct position arrays reachable by any sequence of placements, sum
//   score(p) = sum of a[p_i], modulo 998244353. n, sum n <= 2e5, a_i <= 1e9. Multitest.
// Solution: a reachable array fills an interval of length l, one person per cell plus the
//   n - l extras as pairs on any cells and at most one singleton on each end (by parity):
//   interior cells hold odd counts. For each l >= 2, stars and bars give the number of
//   arrays C(l - 1 + k, k) and one cell's total pair count C(l - 1 + k, l), hence a weight
//   per interior cell (mid_w) and an end bonus. Over the n - l + 1 shifts a cell is
//   interior a trapezoid number of times, summed in O(1) from prefix sums of a and c*a[c]:
//   O(n) per test after factorials; l = 1 adds n * sum(a).

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
