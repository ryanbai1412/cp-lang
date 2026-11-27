// expect: ok, output "3\n1 2 3\n2\nNO"
// `D.all` is the domain's own member, every position in walk order; a value's `s.all(p)` is a
// different member, so the two don't collide.
domain E = 1..=3
fn n<A: Domain>() -> Int = A.all.len
print(E.all.len)
print(E.all)
print(n<E>() - 1)
print(yes(E.all.all(e => e.ord > 0)))
