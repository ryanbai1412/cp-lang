use std.segtree.{MergeSortTree}

let n = 50_000
let q = 100_000
domain I = 0..n
var seed = 123456789
fn next_rng() -> Int:
    seed = (seed * 1103515245 + 12345) % 2147483648
    seed

let a = [I] _ => next_rng() % 1_000_000
let mst = MergeSortTree<I, Int>(a)
var acc = 0
for _ in 0..q:
    let x = next_rng() % n
    let y = next_rng() % n
    let lo = min(x, y)
    let hi = max(x, y)
    let v = next_rng() % 1_000_000
    acc += mst.count_le(I(lo)..=I(hi), v)
print(acc)
