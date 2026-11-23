// The annotation widens a fresh value's range cells (decision 366): `a` stores Ints, so the large `+=` can't overflow the read's range.
let n = read<1..=5>()
var a: [n]Int = read<[n]0..=5>()
for i in a.keys: a[i] += 4000000000
var b: [n]Int = [n] i => 3
b[0] += 5000000000
print(a.sum(), b.sum())
