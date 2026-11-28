// expect: ok, output "1 1"
// engines: next
domain A = 0..2
domain B = 0..3
let p = (A, B).nth(4)
print(Int(p._0), Int(p._1))
