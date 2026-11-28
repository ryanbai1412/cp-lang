// expect: ok, output "1 0 1 2"
// engines: next
domain A = 0..2
domain B = 0..3
let p = (A, B, A, B).nth(23)
print(Int(p._0), Int(p._1), Int(p._2), Int(p._3))
