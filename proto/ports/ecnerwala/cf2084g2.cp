// Codeforces 2084 G2: Wish Upon a Satellite (Hard Version).
// Concave slope-trick DP: adding (2c - k)(2(half - c) - (n - k)) shifts slope[j]
// by (a1 - a0) - 8j; a free value inserts a 0 slope in descending order.
@!multitest
use std.lazy.LazyTreap

struct Affine(base: Int, step: Int)

let n: 1..=500_000 = read()
domain Pos = 1..=n
let a: [Pos]?(1..=n) = read(none: 0)
let half = n / 2

var fixed_even: [1..=n]?Bool = none
for i in Pos:
    if let v = a[i]:
        fixed_even[v] = i.label % 2 == 0

var slopes = LazyTreap<Int, Affine>(
    apply: (v, left_sz, f) => v + f.base + left_sz * f.step,
    compose: (f, g) => Affine(f.base + g.base, f.step + g.step),
    split: (f, left_sz) => (f, Affine(f.base + (left_sz + 1) * f.step, f.step)),
    before: (x, y) => x >= y,
)

fn gain(k: Int, c: Int) -> Int = (2 * c - k) * (2 * (half - c) - (n - k))

var c0 = 0
var v0 = 0
for k in 0..n:
    let a0 = gain(k, c0)
    let a1 = gain(k, c0 + 1)
    v0 += a0
    slopes.apply_all(Affine(a1 - a0, -8))
    if let is_even = fixed_even[k + 1]:
        c0 += Int(is_even)
    else:
        slopes.insert(0)

let best = v0 + slopes.take(half - c0).sum()
print((best + n * (n + 1) * (n + 1) / 2) / 2)
