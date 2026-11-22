// expect: ok, output "4\n0"
// rule: 04.2.17
domain V = 1..=4
print(Int(V.cuts(4)))
print(Int(V.cuts.at(5) ?? V.cuts(0)))
