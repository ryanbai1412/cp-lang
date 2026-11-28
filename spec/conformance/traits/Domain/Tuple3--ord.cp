// expect: ok, output "17"
// engines: next
domain A = 0..2
domain B = 0..3
domain C = 0..5
let p: (A, B, C) = (A(1), B(0), C(2))
print(p.ord)
