// expect: ok, output "1 0 2"
// engines: next
domain A = 0..2
domain B = 0..3
domain C = 0..5
let p = (A, B, C).nth(17)
print(Int(p._0), Int(p._1), Int(p._2))
