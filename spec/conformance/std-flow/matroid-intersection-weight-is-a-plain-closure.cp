// expect: ok, output "2 2 8"
// `weight` is a plain `E -> Int` with a `_ => 0` default, so a caller passes the closure
// itself or leaves it out; with it, the largest common independent set is the heaviest one.
use std.flow.{GraphicMatroid, ColorMatroid, matroid_intersection}

domain V = 0..3
domain E = 0..3
domain C = 0..3
let es: [E](u: V, v: V) = [E] e => [
    (u: V(0), v: V(1)),
    (u: V(1), v: V(2)),
    (u: V(0), v: V(2)),
][e.ord]
let wt: [E]Int = [E] e => [1, 5, 3][e.ord]
let colors: [E]C = [E] e => C(e.ord)
let limits: [C]Int = [C] _ => 1
let plain = matroid_intersection<E>(GraphicMatroid(es), ColorMatroid(colors, limits))
let heavy = matroid_intersection<E>(
    GraphicMatroid(es),
    ColorMatroid(colors, limits),
    weight: e => wt[e],
)
print(
    E.all.count(e => plain[e]),
    E.all.count(e => heavy[e]),
    E.all.sum(e => if heavy[e]: wt[e] else: 0),
)
