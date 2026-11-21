use std.beats.{BeatsSegTree}

let n = 50_000
let q = 100_000
domain I = 0..n
var seed = 123456789
fn next_rng() -> Int:
    seed = (seed * 1103515245 + 12345) % 2147483648
    seed

let init = [I] _ => (next_rng() % 200_001) - 100_000
var seg = BeatsSegTree<I>(init)
var acc = 0
for step in 0..q:
    let a = next_rng() % n
    let b = next_rng() % n
    let lo = min(a, b)
    let hi = max(a, b)
    let r = I(lo)..=I(hi)
    let op = step % 4
    if op == 0:
        seg.chmin(r, (next_rng() % 200_001) - 100_000)
    elif op == 1:
        seg.chmax(r, (next_rng() % 200_001) - 100_000)
    elif op == 2:
        seg.add(r, (next_rng() % 201) - 100)
    else:
        acc += seg.sum(r)
print(acc)
