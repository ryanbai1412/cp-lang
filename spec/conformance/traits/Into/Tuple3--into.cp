// expect: ok, output "7"
// engines: next
domain A = 0..2
domain B = 0..3
let p: (A, B, A) = (1, 0, 1).into()
print(Int(p))
