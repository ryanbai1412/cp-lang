use std.timetree.{TimeTree}

let n = 30_000
let m = 60_000
let q = 30_000
domain V = 0..n
var seed = 123456789
fn next_rng() -> Int:
    seed = (seed * 1103515245 + 12345) % 2147483648
    seed

var tt = TimeTree<(V, V)>(q)
for _ in 0..m:
    let a = next_rng() % q
    let b = next_rng() % q
    let lo = min(a, b)
    let hi = max(a, b) + 1
    let u = V(next_rng() % n)
    let v = V(next_rng() % n)
    tt.add_interval(lo, hi, (u, v))

var dsu = Dsu<V>()
let res = tt.run(
    mark: () => dsu.mark(),
    apply: e => dsu.unite(e.0, e.1),
    answer: _ => dsu.parts,
    undo: snap => dsu.undo_to(snap),
)
var acc = 0
for x in res: acc += x
print(acc)
