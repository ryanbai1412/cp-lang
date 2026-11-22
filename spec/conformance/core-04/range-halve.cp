// expect: ok, output "23\n2\n2"
// rule: 04.4.17 04.7.3 04.8.7
domain V = 0..5
let r = V.range
let (x, y) = r.halve()!
print(x.len * 10 + y.len)
print(Int(r.mid))
print(Int(r.mid.r!) - Int(y.first!) + Int(x.r))
