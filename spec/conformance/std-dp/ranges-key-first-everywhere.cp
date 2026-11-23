// expect: ok, output "4 4 6"
// input: "3\n4 5 6\n"
// A `V.ranges` key is non-empty wherever it is bound: a `for` over `V.ranges`, a `V.ranges` parameter and a `[V.ranges]` table's key all give `first`/`last` as `V` (lib/search-dp.md).
let n: 1..=5 = read()
domain V = 0..n
let a: [V]Int = read()
fn g(r: V.ranges) -> Int = a[r.first]
let t = [V.ranges] r => a[r.last]
for r in V.ranges:
    let f: V = r.first
    if r.len == n: print(a[f], g(r), t[r])
