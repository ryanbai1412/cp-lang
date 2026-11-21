use std.persistent.{RangeKth}

let n = 50_000
let q = 100_000
domain I = 0..n
var seed = 123456789
fn next_rng() -> Int:
    seed = (seed * 1103515245 + 12345) % 2147483648
    seed

let a = [I] _ => next_rng() % 1_000_000
let rk = RangeKth<I, Int>(a)
var acc = 0
for _ in 0..q:
    let x = next_rng() % n
    let y = next_rng() % n
    let lo = min(x, y)
    let hi = max(x, y)
    let k = (hi - lo) / 2
    acc += rk.kth(I(lo)..=I(hi), k)
print(acc)
