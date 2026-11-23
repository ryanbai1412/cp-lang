// expect: ok, output "0 1000000000 0 1000000000"
// rule: 03.1.8
// `List.filled(n, inf)` with `let inf = 1000000000` holds `Int`s, as `[inf, inf]` does: a later `c[0] = 0` is fine.
domain V = 0..3
let inf = 1000000000
var c = List.filled(3, inf)
c[0] = 0
var d = [V].filled(inf)
d[V(0)] = 0
print(c[0], c[1], d[V(0)], d[V(2)])
