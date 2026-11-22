// expect: ok, output "4\n4\n543224\n25\n3\n2"
// rule: 04.4.12 04.4.13 04.4.14
domain V = 1..=6
let r = V(2)..=V(5)
print(r.len)
var c = 0
for v in V:
    if v in r: c += 1
print(c)
var s = 0
for v in r.rev: s = s * 10 + Int(v)
for v in r.step(2): s = s * 10 + Int(v)
print(s)
print(Int(r.first!) * 10 + Int(r.last!))
let k = V.cuts(0)..V.cuts(3)
print(k.len)
print(Int(k.last!))
