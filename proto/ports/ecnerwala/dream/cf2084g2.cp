// cf2084g2: concave slope-trick optimization of cf2084g1.
// Adding (2c - k)(2(half - c) - (n - k)) shifts slope[j] by (b - a) - 8j;
// a free value takes max(dp[c], dp[c - 1]), inserting a 0 slope in descending order.
--- with multitest() ---

domain Pos = read(1..=5e5)
let n = Pos.count
let half = n / 2
let a: [Pos]?(1..=n) = read(none: 0)
let fixed_even: [1..=n]?Bool = a.invert().map(i => i.ord % 2 == 1)

struct Affine(base: Int, step: Int)

// Concave DP represented as base value at c0 plus descending slopes.
var c0 = 0
var v0 = 0
var slopes = LazyTreap<Int, Affine>(
    order: desc,
    apply: (v, left_sz, f) => v + f.base + left_sz * f.step,
    split: (f, left_sz) => (f, Affine(f.base + (left_sz + 1) * f.step, f.step)),
    compose: (f, g) => Affine(f.base + g.base, f.step + g.step),
)

fn gain(k: Int, c: Int) -> Int = (2 * c - k) * (2 * (half - c) - (n - k))

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
