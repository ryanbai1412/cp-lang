// expect: ok, output "27\n9\n24"
// rule: 07.7.4
// A reducer over a `yields` getter, whose closure reads a `let` from outside,
// runs on the getter's values as it yields them; a `var` it reads still counts.
domain B = 0..8
let s = Subset<B>.single(B(1)).with(B(3)).with(B(5))
var t = 0
for k in 1..=2:
    t += s.items.sum(of: b => b.ord * k)
print(t)
print(s.items.fold((a, b) => a + b.ord, 0))
var w = 2
print(s.items.sum(of: b => b.ord * w + w))
