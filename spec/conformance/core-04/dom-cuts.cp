// expect: ok, output "5\n0\n4"
// rule: 04.2.5 04.2.16
domain V = 1..=4
print(V.cuts.count)
print(Int(V.cuts.first!))
print(Int(V.cuts.last!))
