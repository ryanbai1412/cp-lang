// expect: ok, output "3\n5\n4\n2\n3\n6\n2 5 0"
// A `yields` std method called on a receiver is a typed Seq, and argmin/argmax take the index from S's Keyed impl (R07.3.1).
let xs: List<Int> = [1, 2, 3]
for (a, b) in xs.cyclic_pairs(): print(min(a, b) + max(a, b))
for (a, b) in xs.pairs(): print(a * b)
domain R = 0..3
let t: [R]Int = [R] r => 5 - r.ord
let m = t.argmin()!
let i: ?R = t.argmax()
print(m.ord, t[m] + xs.argmax()!, i!.ord)
