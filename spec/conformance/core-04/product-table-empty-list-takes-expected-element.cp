// expect: ok, output "0\n1"
// rule: 04.2.11
// A table over an inline product takes its cells' type from where it is stored, as one over a single domain does: `[]` is an empty `List<Arc>`.
struct Arc(to: Int, w: Int)
struct G<V>(out: [V]List<Arc>)
domain A = 0..2
domain B = 0..3
let g = G<(A, B)>([(A, B)] v => [])
print(g.out[(A(1), B(2))].len)
var h: [A, B]List<Arc> = [A, B] (a, b) => []
h[A(0), B(1)].push(Arc(to: 1, w: 2))
print(h[A(0), B(1)].len)
