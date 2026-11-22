// expect: ok, output "4\n4"
// rule: 04.2.16
domain P = 0..4
domain V = 1..=4
print(Int(P.cuts.last!))
print(Int(V.cuts.last!))
