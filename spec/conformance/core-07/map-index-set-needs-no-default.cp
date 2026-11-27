// expect: ok, output "4 1 | 9 2"
// rule: R07.12.9
// `m[k] = v` is `Map`'s `IndexSet` write: it inserts or replaces, so a value type with no `Default` can be stored; `m[k] op= v` is the one that starts an absent key at `V.default()`.
struct P(a: Int)
var m = Map<Int, P>()
m[1] = P(3)
m[1] = P(4)
var c = Map<Str, Int>()
c["x"] += 4
c["x"] += 5
c["y"] = 1
print(m[1].a, m.len, "|", c["x"], c.len)
