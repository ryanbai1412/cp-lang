use std.lazy.LazyTreap

struct Affine(base: Int, step: Int)

let n = 200_000
var slopes = LazyTreap<Int, Affine>(
    apply: (v, left_sz, f) => v + f.base + left_sz * f.step,
    compose: (f, g) => Affine(f.base + g.base, f.step + g.step),
    split: (f, left_sz) => (f, Affine(f.base + (left_sz + 1) * f.step, f.step)),
    before: (x, y) => x >= y,
)

var x: Int = 1234567
for i in 1..=n:
    x = (x * 1103515245 + 12345) & 0x7fffffff
    let base = (x % 1000) - 500
    let step = -(i % 7)
    slopes.apply_all(Affine(base, step))
    slopes.insert(base * 2 + step)

print(slopes.take(n / 2).sum())
