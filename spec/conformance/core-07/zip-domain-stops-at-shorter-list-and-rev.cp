// expect: ok, output "10 20 -20 -10\n2 4\n5 3 1"
// rule: 07.4.10
// `zip` with a domain pairs by walk order and stops at the shorter side, whichever argument it is and also against a reversed domain; step views compose with `drop` and `.rev`.
domain R = 0..4
let xs = [10, 20]
var t: [R]Int = 0
for (r, x) in zip(R, xs):
    t[r] += x
for (x, r) in zip(xs, R.rev):
    t[r] -= x
print(t)
let p = [0, 1, 2, 3, 4, 5]
print([x for x in p[1..].odd])
print([x for x in p.drop(1).even.rev])
