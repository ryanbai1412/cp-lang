// expect: ok, output "4"
// input: "1 1 1 1"
domain V = 0..3
let a: [V.cuts]Int = read()
print(a.sum())
