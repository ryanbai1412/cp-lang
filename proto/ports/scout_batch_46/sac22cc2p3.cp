// SAC '22 CC2 P3 — original: corpus/submissions/4121049.cpp
// Idiomatic CP Lang v2 (locked spec: docs/one-page-cheat-sheet.md)

let (k, r): (1..=19, 0..=1_000_000_000) = read()
let n = 2 ** k
domain V = 1..=2 * n - 1

// each line is a fork at u: take a for w, or take b for free
var fork: [V]?(a: V, b: V, w: Int) = none
for _ in 0..n - 1:
    let (u, a, b, w): (V, V, V, Int) = read()
    fork[u] = (a, b, w)

// FRICTION 46.9: @memo recursion goes 2^19 deep on a chain; the spec gives no depth guarantee
@memo
fn best(u: V) -> Int:
    if let f = fork[u]: max(f.w + best(f.a), best(f.b)) else: 0

print(r + best(V.first))
