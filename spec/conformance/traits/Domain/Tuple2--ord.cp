// expect: ok, output "5"
// engines: next
domain A = 0..2
domain B = 0..3
let p: (A, B) = (A(1), B(2))
print(p.ord)
