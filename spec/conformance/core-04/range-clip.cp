// expect: ok, output "24"
// rule: 04.4.16
domain V = 1..=6
let r = (V(2)..=V(5)).clip(V(4)..=V(6))
print(r.len * 10 + Int(r.first!))
