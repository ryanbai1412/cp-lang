// expect: ok, output "1 2 5"
// engines: next
domain A = 0..2
domain B = 0..3
let p: (A, B) = (1, 2).into()
print(Int(p._0), Int(p._1), Int(p))
