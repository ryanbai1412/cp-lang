// expect: ok, output "1 3"
domain V = 0..4
let p = Pair<V>.nth(4)
print(Int(p.lo), Int(p.hi))
