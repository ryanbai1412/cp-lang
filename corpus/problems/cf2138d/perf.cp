// Codeforces 2138 D, tuned: port.cp with the inverses 1/k for k <= q read
// from a table made once per test, as orig.cpp's inv_val, instead of
// `comb.inv` (two loads and a product) inside the O(n q) loop, and the
// span length times the count taken as one Int before the reduction.
//
// Codeforces 2138 D: Antiamuny Wants to Learn Swap.
// `n` sliders sit at strictly increasing positions `raw_a[i]` in `1..=l`; each
// operation `(i, x)` moves slider `i` to `x`, pushing neighbors to stay
// strictly ordered. Shifting by `- 1 - i` (`a[i] = raw_a[i] - 1 - i`, op
// `(val: x - i, pos: i - 1)`) turns strict gaps into non-decreasing order.
//
// For fixed slider `i` and threshold `y`, an op with `pos <= i, val >= y`
// pushes slider `i` to `>= y` (`num_right` ops), while `pos >= i, val < y`
// pushes it to `< y` (`num_left` ops). In a random order of the `q` ops,
// whichever of these `num_left + num_right` ops runs last decides `[a'[i] >= y]`,
// giving probability `num_right / (num_left + num_right)` (or `[a[i] >= y]` if
// none). Integrating over `y in [0, l]` yields `E[a'[i]]`; adding back `i + 1`
// and scaling by `q!` sums final positions over all `q!` operation orders.

type Z = Mod<1_000_000_007>
let comb = Comb<Z>()

fn span_expect(lo: Int, hi: Int, ai: Int, num_left: Int, num_right: Int, inv: List<Z>) -> Z:
    if hi <= lo: return 0
    if num_left + num_right == 0: Z(min(max(ai, lo), hi) - lo)
    else: Z((hi - lo) * num_right) * inv[num_left + num_right]

@!multitest
let (n, l, q): (1..=5000, 1..=1_000_000_000, 1..=5000) = read()
domain I = 0..n
let raw_a: [I]Int = read()
let a = [I] i => raw_a[i] - 1 - i.ord
let raw_ops: [0..q](Int, Int) = read()
let ops = [(val: x - i, pos: I(i - 1)) for (i, x) in raw_ops].sorted()

let fact_q = comb.fact(q)
let inv = [if k == 0: Z(0) else: comb.inv(k) for k in 0..=q]
var out: [I]Z = 0
for i in I:
    var num_left = 0
    var num_right = [1 for (_, pos) in ops if pos <= i].sum()
    var ans: Z = 0
    var lo = 0
    for (hi, pos) in ops:
        ans += span_expect(lo, hi, a[i], num_left, num_right, inv)
        lo = hi
        if pos <= i: num_right -= 1
        if pos >= i: num_left += 1
    ans += span_expect(lo, l, a[i], num_left, num_right, inv)
    out[i] = (ans + Z(i.ord + 1)) * fact_q

print(out)
