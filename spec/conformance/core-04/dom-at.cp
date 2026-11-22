// expect: ok, output "3\n1"
// rule: 04.2.7 04.2.8 04.8.9
domain V = 1..=4
print(Int(V.at(3) ?? V(1)))
print(Int(V.at(9) ?? V(1)))
