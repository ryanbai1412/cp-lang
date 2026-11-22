// expect: ok, output "12\n23"
// rule: 04.2.16 04.3.1
domain V = 1..=4
domain P = 0..4
print(Int(V(2).l) * 10 + Int(V(2).r))
print(Int(P(2).l) * 10 + Int(P(2).r))
