// expect: ok, output "26"
// rule: 04.2.3
domain V = 2..=6
print(Int(V.first!) * 10 + Int(V.last!))
