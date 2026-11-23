// expect: ok, output "2"
// A closure taking a `SegTree` reads its `all` getter in both engines; the C++ used to bind the read-only parameter to a non-const reference (rebuild round 33 F6).
domain G = 0..3
var t = SegTree<G, Int>(min, fill: 5)
t.set(G(1), 2)
let f = (s: SegTree<G, Int>) => s.all
print(f(t))
