// expect: ok, output "2 0"
// rule: 04.2.13
// `a.ring.step(i, d)` steps `d` positions of `a`, wrapping both ways.
let a = [1, 2, 3]
print(a.ring.step(0, -1), a.ring.step(2, 1))
