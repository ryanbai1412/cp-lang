// expect: ok, output "E\nS N\nAdd Neg\nmid=W"
// rule: R10.3.5 R08.1.6
// An enum prints its variant's name, a variant with fields too, unless it writes its own impl.
enum D = N | E | S | W
enum Op = Add(a: Int, b: Int) | Neg(Int)
print(D.E)
print([D.S, D.N])
print(Op.Add(1, 2), Op.Neg(3))
print("mid={D.W}")
