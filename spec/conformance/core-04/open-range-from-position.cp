// expect: ok, output "3\n4\n5"
// rule: 04.4.10
// `i..` is `i..=D.last` outside a slice too (rebuild round 18 H08).
domain V = 0..6
for j in V(3)..: print(j.ord)
