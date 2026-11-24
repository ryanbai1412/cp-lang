// expect: ok, output "2 1"
// rule: 07.9.3
// A generator expression passed to the free `max(.., by:)` has its element's type: the `by:` closure takes that element and the `?T` result unwraps to it, so `top.dx` is an Int.
let seen = [(t: 0, x: 0), (t: 2, x: 3), (t: 3, x: 5)]
let top = max((dx: b.x - a.x, dt: b.t - a.t) for (a, b) in seen.adjacent, by: s => s.dx * 10 / s.dt)!
print(top.dx, top.dt)
