// A field holding an operation value is called like the operation, and a
// `let` receiver may call it: `s.add(3, 4)` is `min(3, 4)` (R05.4.1).
let s = Semiring(min, +)
struct G<A>(op: A)
let g = G(max)
print(s.add(3, 4), s.mul(3, 4), g.op(3, 4))
