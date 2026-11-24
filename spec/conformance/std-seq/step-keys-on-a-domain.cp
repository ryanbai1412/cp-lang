// expect: ok, output "1 0 3 50"
// rule: 04b.1.10 09.3.10
// A step view over a domain-keyed array has the domain keys the stride visits (adversarial round 47).
domain V = 1..=6
let a: [V]Int = [V] v => Int(v) * 10
let s = a.step(2)
print(Int(V(3) in s.keys), Int(V(4) in s.keys), s.keys.len, s.at(V(5)))
