// expect: ok, output "YES"
let b = gen<Bool>()
let c: Bool = gen()
print(yes((b or not b) and (c or not c)))
