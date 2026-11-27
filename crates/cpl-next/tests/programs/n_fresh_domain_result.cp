// expect: ok, output "3 3 2 2\n6 2"
// rule: R02.9.3 R02.9.4
fn evens(n: Int) -> (domain I, [I]Int):
    domain J = 0..n
    (J, [J] j => Int(j) * 2)
let (domain A, a) = evens(3)
let (domain B, b) = evens(2)
print(a.len, A.count, b.len, B.count)
var s = 0
for x in a: s += x
var t = 0
for x in b: t += x
print(s, t)
