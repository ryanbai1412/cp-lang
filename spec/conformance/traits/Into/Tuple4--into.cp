// expect: ok, output "11"
// engines: next
domain A = 0..2
let p: (A, A, A, A) = (1, 0, 1, 1).into()
print(Int(p))
