// expect: ok, output "3 6\n0 3\n1"
// rule: R04.3.4
// An axis ending at a domain's count, `[0..V.count - 1]`, is a range type outside a body too:
// a type alias, a result and a parameter hold it, and each domain gives it its own count.
type Path<V> = [0..V.count - 1](u: V, v: V)
fn path<V: Domain>() -> Path<V> = [0..V.count - 1] i => (u: V.nth(i), v: V.nth(i + 1))
fn total<V: Domain>(p: Path<V>) -> Int:
    var s = 0
    for e in p: s += e.v.ord
    s
domain V = 0..4
let p = path<V>()
print(p.len, total(p))
print(p[0].u.ord, p[2].v.ord)
domain W = 0..2
print(path<W>().len)
