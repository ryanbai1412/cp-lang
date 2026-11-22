// expect: ok, output "14"
// rule: 04.4.15
domain V = 1..=6
let r = V(2)..=V(4)
print(Int(r.l) * 10 + Int(r.r))
