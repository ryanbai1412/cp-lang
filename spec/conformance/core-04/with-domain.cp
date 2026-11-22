// expect: ok, output "3\n1"
// rule: 04b.4.4
let (domain I, b) = [3, 1, 2].with_domain()
print(I.count)
print(b[I(1)])
