// expect: ok, output "3"
// input: "1 1 1"
let (domain D, comp) = compress([30, 10, 20, 10])
let a: [D]Int = read()
print(a.sum())
