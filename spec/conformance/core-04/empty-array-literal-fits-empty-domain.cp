// expect: ok, output "0 0"
// rule: 04.8.10
let a: [0..0]Int = []
let b: [0..0](u: Int, v: Int) = []
print(a.len, b.len)
