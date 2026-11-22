// Codeforces 2157 C — Meximum Array 2
//
// Constraints have `n, k, q <= 100`, so marking each query range directly is
// O(n * q) <= 10^4. Every position in a `Min` range must be `>= k`, and every
// position in a `Mex` range cannot equal `k` (while the range must contain all
// of `0..k`):
// - Positions in `Min` only (or unconstrained) get `k`.
// - Positions in both `Min` and `Mex` get `k + 1` (`>= k` and `!= k`).
// - Positions in `Mex` only cycle through `0..k` (`cnt % k`), guaranteeing
//   every valid `Mex` range covers `0..k`.

enum Kind:
    Min
    Mex

@!multitest
let (n, k, q): (1..=100, 1..=100, 1..=100) = read()
domain Pos = 1..=n
var in_min: [Pos]Bool = false
var in_mex: [Pos]Bool = false

for _ in 0..q:
    let (c, l, r): (1..=2, Pos, Pos) = read()
    let kind = if c == 1: Kind.Min else: Kind.Mex
    for i in l..=r:
        match kind:
            Kind.Min => in_min[i] = true
            Kind.Mex => in_mex[i] = true

var a: [Pos]Int = k
var cnt = 0
for i in Pos if in_mex[i]:
    if in_min[i]:
        a[i] = k + 1
    else:
        a[i] = cnt % k
        cnt += 1
print(a)
