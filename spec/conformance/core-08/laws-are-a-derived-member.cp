// expect: ok, output "assoc 3 3 3 YES NO\nsmall 3 YES YES\n2 assoc YES\nident 3 YES\nreplace 3 3 3 YES\ncommute 3 3 3 YES\ncomposes 3 3 3 YES"
// rule: 08.3.5
// A struct with `law`s has a derived `@get laws`: each law by name, the pool size for each parameter, and `holds(samples, picks)`. One parameter type takes a `List<T>` of samples; several take a record keyed by type name.
use std.algebra.{Monoid, Action}
struct M:
    x: Int
    law assoc(a: M, b: M, c: M) = (a.x - b.x) - c.x == a.x - (b.x - c.x)
    law small(a: M) = a.x < 2

let s = [M(0), M(1), M(2)]
for l in M(0).laws:
    print(l.name, l.sizes(s), yes(l.holds(s, [0, 0, 0])), yes(l.holds(s, [1, 1, 1])))
let m = Monoid<Int>.of(+)
print(m.laws.len, m.laws[0].name, yes(m.laws[0].holds([1, 2, 3], [0, 1, 2])))
let act = Action<Int, Int>(id: 0, compose: (f, g) => f + g, apply: (f, v, n) => v + f * n, over: Monoid<Int>.of(+))
let ss = (T: [1, 5, 7], F: [0, 3, 4])
for l in act.laws: print(l.name, l.sizes(ss), yes(l.holds(ss, [1, 2, 0])))
