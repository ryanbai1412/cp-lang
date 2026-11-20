// ETSR '23 Round 4 P2 — Extended Euler Totient Power Tower (CP Lang v2 spec)
// original: corpus/submissions/5529698.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca) + stdlib/lazy.cp + stdlib/mod.cp
//
// Range update `a[i] := c^(a[i])` and range sum mod `p` (runtime modulus).
// Precompute the totient chain [p, phi(p), ..., 1] and each element's tower
// up to stabilization, then serve range shifts with `LazySegTree`.
use std.math.{factor}

fn phi(x: Int) -> Int:
    var ans = x
    for (p, _) in factor(x):
        ans = ans / p * (p - 1)
    ans

fn pow_capped(base: Int, exp: Int, m: Int) -> (over: Bool, val: Int):
    if m == 1: return (true, 0)
    var (over, b_over) = (false, false)
    var (ans, b, e) = (1, base, exp)
    while e > 0:
        if (e & 1) == 1:
            over = over or b_over or ans * b >= m
            ans = (ans * b) % m
        b_over = b_over or b * b >= m
        b = (b * b) % m
        e >>= 1
    (over, ans)

let (n, q, p, base): (1..=5e4, 1..=5e4, 1..=1e8, 1..=1e8) = read()
type Z = Mod<p>
domain I = 1..=n
let a: [I]Int = read()

var phis: List<Int> = []
var cur_m = p
while true:
    phis.push(cur_m)
    if cur_m == 1: break
    cur_m = phi(cur_m)

let K = phis.len + 1
domain Slot = 0..K

fn tower(steps: Int, x: Int, depth: Int) -> (over: Bool, val: Int):
    if steps == 0: return (x >= phis[depth], x % phis[depth])
    let next_d = min(phis.len - 1, depth + 1)
    let (of, nx) = tower(steps - 1, x, next_d)
    let exp = if of: nx + phis[next_d] else: nx
    let (cf, ans) = pow_capped(base, exp, phis[depth])
    (of or cf, ans)

// FRICTION ET6: Fixed-length small array `[Slot]Z` stored inside a
// `LazySegTree` node needs copy/value semantics for `Monoid` and `Action`,
// and clamping shift `s + shift` into `Slot` drops to raw `.to_idx` math.
let node_m = Monoid<[Slot]Z>([Slot] _ => Z(0), (x, y) => [Slot] s => x[s] + y[s])
let shift_act = Action<[Slot]Z, Int>(
    0,
    (a, b) => min(a + b, K - 1),
    (k, x, _) => [Slot] s => x[Slot(min(s.to_idx + k, K - 1))],
    over: node_m,
)

var st = LazySegTree<I, [Slot]Z, Int>(
    shift_act,
    [I] i => [Slot] s => Z(tower(s.to_idx, a[i], 0).val),
)

for _ in 0..q:
    let (ty, l, r): (0..=1, I, I) = read()
    if ty == 0: st.apply(l..=r, 1)
    else: print(st.fold(l..=r)[Slot.first])
