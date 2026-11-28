// expect: ok, output "31"
// engines: next
domain A = 0..2
domain B = 0..3
let p: (A, B, A, B) = (A(1), B(2), A(0), B(1))
print(p.ord)
