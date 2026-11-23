// A closure parameter of a struct with cached fields (a `SegTree`) is its
// own copy, as a fn parameter is: its getters take it by reference, so
// the C++ can't hand them a `const` one (rebuild round 33 F6).
domain G = 0..3
var t = SegTree<G, Int>(min, fill: 5)
t.set(G(1), 2)
let f = (s: SegTree<G, Int>) => s.all
print(f(t))
let ok = (s: SegTree<G, Int>, k: Int) => s.all <= k
print(yes(ok(t, 1)), yes(ok(t, 2)))
t.set(G(1), 7)
print(f(t), t.all)
