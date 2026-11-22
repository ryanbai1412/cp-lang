// expect: ok, output "6"
// input: "1 1 1 1 1 1"
domain R = 0..2
domain C = 0..3
domain D = (R, C)
let a: [D]Int = read()
print(a.sum())
