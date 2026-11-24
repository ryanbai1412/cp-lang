// n: 1024
// let: r = \sqrt n; V = \sqrt n
// fixed: oracle
use std.flow.{GraphicMatroid, ColorMatroid, matroid_intersection}
// Spanning forests of the complete graph on sqrt n vertices, at most one edge per colour.
let n: Int = read()
let k = n.isqrt
domain V = 0..k
domain E = 0..k * k
domain C = 0..k
let es: [E](u: V, v: V) = [E] e => (u: V(e.ord / k), v: V(e.ord % k))
let col = [E] e => C(e.ord % k)
let lim = [C] _ => 1
print(matroid_intersection<E>(GraphicMatroid(es), ColorMatroid(col, lim)).count(x => x)) // timed: flow.matroid_intersection
