// expect: ok, output "3\n0\n3\n3\n0"
// rule: 04.4.18 04.4.17 04.7.1 04.7.2 04.8.7
domain V = 0..5
let e = V(3)..V(3)
print(Int(e.mid))
var n = 0
if e.mid.r! in e: n += 1
print(n)
let one = V(3)..=V(3)
print(Int(one.mid))
print(Int(one.mid.r!))
var h = 0
if let p = one.halve(): h = 1
print(h)
