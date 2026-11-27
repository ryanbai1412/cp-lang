// expect: R0101
// rule: R04.3.4
// A key past an axis that ends at a domain's count traps: `[0..V.count - 1]` has `V.count - 1` keys.
type Path<V> = [0..V.count - 1](u: V, v: V)
fn path<V: Domain>() -> Path<V> = [0..V.count - 1] i => (u: V.nth(i), v: V.nth(i + 1))
fn at<V: Domain>(p: Path<V>, k: Int) -> Int = p[k].u.ord // <- here
domain V = 0..4
let p = path<V>()
print(at(p, 2))
print(at(p, 3))
