// expect: ok, output "8"
// input: "1 1 1 1 1 1 1 1"
domain B = 0..3
let a: [Subset<B>]Int = read()
print(a.sum())
