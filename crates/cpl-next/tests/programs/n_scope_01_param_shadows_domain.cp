// expect: ok, output "2\n0"
// A program domain named like a std fn's type parameter does not leak into that fn.
domain D = 0..4
print(D.count(x => x == D(1) or x == D(2)))
let a: [D]Int = [D] v => Int(v)
print(a.first!)
